use ractor::{concurrency::Duration, Actor, ActorProcessingErr, ActorRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use tracing::{debug, error, info, instrument, warn};

// Re-export common types that clients need
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Country {
    pub code: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct City {
    pub city: String,
}

pub type CountryCitiesResult =
    Result<(Country, Vec<City>), (Country, Box<dyn std::error::Error + Send + Sync>)>;

// Abstract iterator trait for city fetching
pub trait CityIterator: Send {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<CountryCitiesResult>> + Send + '_>>;
    fn progress(&self) -> (usize, usize);
}

// Trait for providing city API functionality
#[async_trait::async_trait]
pub trait CityApiProvider: Send + Sync + Clone + 'static {
    type Iterator: CityIterator;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get_all_cities(&self) -> Result<Self::Iterator, Self::Error>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CityEntry {
    pub city_code: String,
    pub city_name: String,
    pub country_name: String,
    pub country_code: String,
    pub image_url: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub struct PeriodicCityUpdater<T: CityApiProvider> {
    update_interval: Duration,
    heartbeat_interval: Duration,
    cities_file_path: String,
    api_provider: T,
}

impl<T: CityApiProvider> PeriodicCityUpdater<T> {
    pub fn new(
        update_interval: Duration,
        heartbeat_interval: Duration,
        cities_file_path: String,
        api_provider: T,
    ) -> Self {
        Self {
            update_interval,
            heartbeat_interval,
            cities_file_path,
            api_provider,
        }
    }

    pub fn with_intervals_secs(
        update_interval_secs: u32,
        heartbeat_interval_secs: u32,
        cities_file_path: String,
        api_provider: T,
    ) -> Self {
        Self {
            update_interval: Duration::from_secs(update_interval_secs as u64),
            heartbeat_interval: Duration::from_secs(heartbeat_interval_secs as u64),
            cities_file_path,
            api_provider,
        }
    }
}

impl<T: CityApiProvider + Default> Default for PeriodicCityUpdater<T> {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(3600), // 1 hour default
            heartbeat_interval: Duration::from_secs(60), // 1 minute default
            cities_file_path: "city.json".to_string(),
            api_provider: T::default(),
        }
    }
}

#[derive(Clone)]
pub struct PeriodicCityUpdaterState {
    pub last_update_time: std::time::Instant,
    pub next_update_time: std::time::Instant,
    pub update_count: u64,
    pub heartbeat_count: u64,
}

#[derive(Debug)]
pub enum CityUpdaterMessage {
    UpdateCities,
    Heartbeat,
}

#[ractor::async_trait]
impl<T: CityApiProvider> Actor for PeriodicCityUpdater<T> {
    type Msg = CityUpdaterMessage;
    type State = PeriodicCityUpdaterState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        info!(
            "Starting PeriodicCityUpdater with update_interval: {:?}, heartbeat_interval: {:?}",
            self.update_interval, self.heartbeat_interval
        );

        let now = std::time::Instant::now();

        // Schedule the first heartbeat and update
        myself
            .cast(CityUpdaterMessage::Heartbeat)
            .expect("Failed to send initial heartbeat");
        myself
            .cast(CityUpdaterMessage::UpdateCities)
            .expect("Failed to send initial update");

        Ok(PeriodicCityUpdaterState {
            last_update_time: now,
            next_update_time: now + self.update_interval,
            update_count: 0,
            heartbeat_count: 0,
        })
    }

    #[instrument(skip(self, myself, state), fields(message_type))]
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            CityUpdaterMessage::Heartbeat => {
                tracing::Span::current().record("message_type", "heartbeat");
                state.heartbeat_count += 1;

                let now = std::time::Instant::now();
                let remaining_time = if state.next_update_time > now {
                    state.next_update_time - now
                } else {
                    Duration::from_secs(0)
                };

                info!(
                    heartbeat_count = state.heartbeat_count,
                    update_count = state.update_count,
                    remaining_minutes = remaining_time.as_secs() / 60,
                    remaining_seconds = remaining_time.as_secs() % 60,
                    "City updater heartbeat - Next update in {}m {}s",
                    remaining_time.as_secs() / 60,
                    remaining_time.as_secs() % 60
                );

                // Schedule next heartbeat
                ractor::concurrency::sleep(self.heartbeat_interval).await;
                myself
                    .cast(CityUpdaterMessage::Heartbeat)
                    .expect("Failed to schedule next heartbeat");
            }
            CityUpdaterMessage::UpdateCities => {
                tracing::Span::current().record("message_type", "update_cities");
                state.update_count += 1;
                let now = std::time::Instant::now();

                info!(
                    update_count = state.update_count,
                    "Starting city.json update"
                );

                match self.update_cities_file().await {
                    Ok(stats) => {
                        state.last_update_time = now;
                        state.next_update_time = now + self.update_interval;
                        info!(
                            cities_processed = stats.cities_processed,
                            new_cities_added = stats.new_cities_added,
                            countries_processed = stats.countries_processed,
                            countries_failed = stats.countries_failed,
                            "City.json update completed successfully"
                        );
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to update cities.json");
                    }
                }

                // Schedule next update
                ractor::concurrency::sleep(self.update_interval).await;
                myself
                    .cast(CityUpdaterMessage::UpdateCities)
                    .expect("Failed to schedule next update");
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateStats {
    pub cities_processed: u32,
    pub new_cities_added: u32,
    pub countries_processed: u32,
    pub countries_failed: u32,
}

impl<T: CityApiProvider> PeriodicCityUpdater<T> {
    #[instrument(skip(self))]
    async fn update_cities_file(
        &self,
    ) -> Result<UpdateStats, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Loading existing cities from {}", self.cities_file_path);

        // Load existing cities
        let mut existing_cities: HashMap<String, CityEntry> = match self
            .load_existing_cities()
            .await
        {
            Ok(cities) => cities,
            Err(e) => {
                warn!(error = %e, "Could not load existing cities, starting with empty collection");
                HashMap::new()
            }
        };

        let initial_count = existing_cities.len();
        debug!(
            existing_cities_count = initial_count,
            "Loaded existing cities"
        );

        // Get all cities iterator
        let mut cities_iter = self
            .api_provider
            .get_all_cities()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut stats = UpdateStats {
            cities_processed: 0,
            new_cities_added: 0,
            countries_processed: 0,
            countries_failed: 0,
        };

        // Process each country
        while let Some(result) = cities_iter.next().await {
            let (current, total) = cities_iter.progress();
            debug!(
                progress = format!("{}/{}", current, total),
                "Processing countries"
            );

            match result {
                Ok((country, cities)) => {
                    stats.countries_processed += 1;
                    debug!(
                        country_name = country.name,
                        country_code = country.code,
                        cities_count = cities.len(),
                        "Processing cities for country"
                    );

                    for city in cities {
                        stats.cities_processed += 1;

                        // Generate a city_code if not present (using city name + country code)
                        let city_code = format!("{}_{}", city.city.replace(" ", "_"), country.code);

                        if !existing_cities.contains_key(&city_code) {
                            let city_entry = CityEntry {
                                city_code: city_code.clone(),
                                city_name: city.city,
                                country_name: country.name.clone(),
                                country_code: country.code.clone(),
                                image_url: String::new(),
                                latitude: 0.0,
                                longitude: 0.0,
                            };

                            existing_cities.insert(city_code, city_entry);
                            stats.new_cities_added += 1;
                        }
                    }
                }
                Err((country, error)) => {
                    stats.countries_failed += 1;
                    warn!(
                        country_name = country.name,
                        country_code = country.code,
                        error = %error,
                        "Failed to get cities for country"
                    );
                }
            }
        }

        // Save updated cities
        self.save_cities(existing_cities).await?;

        Ok(stats)
    }

    #[instrument(skip(self))]
    async fn load_existing_cities(
        &self,
    ) -> Result<HashMap<String, CityEntry>, Box<dyn std::error::Error + Send + Sync>> {
        if !Path::new(&self.cities_file_path).exists() {
            debug!("Cities file does not exist, returning empty collection");
            return Ok(HashMap::new());
        }

        let content = tokio::fs::read_to_string(&self.cities_file_path).await?;
        let cities: Vec<CityEntry> = serde_json::from_str(&content)?;

        let mut city_map = HashMap::new();
        for city in cities {
            city_map.insert(city.city_code.clone(), city);
        }

        Ok(city_map)
    }

    #[instrument(skip(self, cities))]
    async fn save_cities(
        &self,
        cities: HashMap<String, CityEntry>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut cities_vec: Vec<CityEntry> = cities.into_values().collect();
        cities_vec.sort_by(|a, b| a.city_name.cmp(&b.city_name));

        let json = serde_json::to_string_pretty(&cities_vec)?;
        tokio::fs::write(&self.cities_file_path, json).await?;

        debug!(
            cities_count = cities_vec.len(),
            file_path = self.cities_file_path,
            "Saved cities to file"
        );

        Ok(())
    }
}
