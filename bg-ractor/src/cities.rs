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
    pub last_update_span_context: Option<tracing::span::Id>,
}

#[derive(Debug)]
pub enum CityUpdaterMessage {
    UpdateCities,
    Heartbeat,
    PrintCityJson,
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

        // Schedule the first heartbeat, update, and city.json print
        myself
            .cast(CityUpdaterMessage::Heartbeat)
            .expect("Failed to send initial heartbeat");
        myself
            .cast(CityUpdaterMessage::UpdateCities)
            .expect("Failed to send initial update");
        myself
            .cast(CityUpdaterMessage::PrintCityJson)
            .expect("Failed to send initial print city.json");

        Ok(PeriodicCityUpdaterState {
            last_update_time: now,
            next_update_time: now + self.update_interval,
            update_count: 0,
            heartbeat_count: 0,
            last_update_span_context: None,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            CityUpdaterMessage::Heartbeat => {
                state.heartbeat_count += 1;

                let now = std::time::Instant::now();
                let remaining_time = if state.next_update_time > now {
                    state.next_update_time - now
                } else {
                    Duration::from_secs(0)
                };

                // Create a comprehensive heartbeat span with proper bg-ractor context
                let heartbeat_span = tracing::debug_span!(
                    "bg_ractor_cities_heartbeat",
                    service.name = "bg_ractor",
                    component = "cities_updater",
                    heartbeat_count = state.heartbeat_count,
                    update_count = state.update_count,
                    next_update_in_secs = remaining_time.as_secs(),
                    last_update_ago_secs = (now - state.last_update_time).as_secs(),
                    last_update_referenced = state.last_update_span_context.is_some(),
                    health.status = "healthy",
                    health.service_uptime_heartbeats = state.heartbeat_count,
                    otel.name = format!("bg_ractor_cities_heartbeat_{}", state.heartbeat_count),
                    otel.kind = "internal",
                    otel.scope.name = "bg-ractor",
                    otel.scope.version = env!("CARGO_PKG_VERSION")
                );

                // Add span link to the last update operation if available
                if let Some(last_update_span_id) = &state.last_update_span_context {
                    debug!(
                        linked_span_id = ?last_update_span_id,
                        "Heartbeat linked to previous update span"
                    );
                }
                let _enter = heartbeat_span.enter();

                info!(
                    heartbeat_count = state.heartbeat_count,
                    update_count = state.update_count,
                    remaining_minutes = remaining_time.as_secs() / 60,
                    remaining_seconds = remaining_time.as_secs() % 60,
                    "bg-ractor cities heartbeat - Next update in {}m {}s",
                    remaining_time.as_secs() / 60,
                    remaining_time.as_secs() % 60
                );

                // Schedule next heartbeat without blocking
                let myself_clone = myself.clone();
                let heartbeat_interval = self.heartbeat_interval;
                tokio::spawn(async move {
                    ractor::concurrency::sleep(heartbeat_interval).await;
                    myself_clone
                        .cast(CityUpdaterMessage::Heartbeat)
                        .expect("Failed to schedule next heartbeat");
                });
            }
            CityUpdaterMessage::UpdateCities => {
                state.update_count += 1;
                let now = std::time::Instant::now();

                // Create a child span for this specific update cycle with proper bg-ractor context
                let update_cycle_span = tracing::info_span!(
                    "bg_ractor_cities_update",
                    service.name = "bg_ractor",
                    component = "cities_updater",
                    update_cycle = state.update_count,
                    otel.name = format!("bg_ractor_cities_update_{}", state.update_count),
                    otel.kind = "internal",
                    otel.scope.name = "bg-ractor",
                    otel.scope.version = env!("CARGO_PKG_VERSION"),
                    cities.processed = tracing::field::Empty,
                    cities.added = tracing::field::Empty,
                    countries.processed = tracing::field::Empty,
                    countries.failed = tracing::field::Empty,
                    update.status = tracing::field::Empty,
                    update.duration_ms = tracing::field::Empty
                );
                let _enter = update_cycle_span.enter();

                info!(
                    update_count = state.update_count,
                    "Starting bg-ractor cities update"
                );

                let start_time = std::time::Instant::now();
                match self.update_cities_file().await {
                    Ok(stats) => {
                        let duration = start_time.elapsed();
                        state.last_update_time = now;
                        state.next_update_time = now + self.update_interval;

                        // Record metrics in the span
                        update_cycle_span.record("cities.processed", stats.cities_processed);
                        update_cycle_span.record("cities.added", stats.new_cities_added);
                        update_cycle_span.record("countries.processed", stats.countries_processed);
                        update_cycle_span.record("countries.failed", stats.countries_failed);
                        update_cycle_span.record("update.status", "success");
                        update_cycle_span.record("update.duration_ms", duration.as_millis() as u64);

                        // Store the span context for future correlation
                        state.last_update_span_context = Some(update_cycle_span.id().unwrap());

                        info!(
                            cities_processed = stats.cities_processed,
                            new_cities_added = stats.new_cities_added,
                            countries_processed = stats.countries_processed,
                            countries_failed = stats.countries_failed,
                            duration_ms = duration.as_millis(),
                            "bg-ractor cities update completed successfully"
                        );
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();

                        // Record error metrics in the span
                        update_cycle_span.record("update.status", "failed");
                        update_cycle_span.record("update.duration_ms", duration.as_millis() as u64);

                        // Store the span context even for failed updates for correlation
                        state.last_update_span_context = Some(update_cycle_span.id().unwrap());

                        error!(
                            error = %e,
                            duration_ms = duration.as_millis(),
                            "Failed to update cities.json in bg-ractor"
                        );
                    }
                }

                // Schedule next update without blocking
                let myself_clone = myself.clone();
                let update_interval = self.update_interval;
                tokio::spawn(async move {
                    ractor::concurrency::sleep(update_interval).await;
                    myself_clone
                        .cast(CityUpdaterMessage::UpdateCities)
                        .expect("Failed to schedule next update");
                });
            }
            CityUpdaterMessage::PrintCityJson => {
                // Create a proper span for city json printing
                let print_span = tracing::debug_span!(
                    "bg_ractor_cities_print",
                    service.name = "bg_ractor",
                    component = "cities_updater",
                    otel.name = "bg_ractor_cities_print",
                    otel.kind = "internal",
                    otel.scope.name = "bg-ractor",
                    otel.scope.version = env!("CARGO_PKG_VERSION")
                );
                let _enter = print_span.enter();

                // Read and print city.json contents
                match self.load_existing_cities().await {
                    Ok(cities) => {
                        info!(total_cities = cities.len(), "Printing city.json contents");

                        // Print first 5 cities as sample
                        let sample_cities: Vec<_> = cities.values().take(5).collect();
                        for (idx, city) in sample_cities.iter().enumerate() {
                            info!(
                                index = idx + 1,
                                city_name = %city.city_name,
                                country_name = %city.country_name,
                                city_code = %city.city_code,
                                "Sample city entry"
                            );
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to read city.json for printing");
                    }
                }

                // Schedule next print (every 20 seconds)
                let myself_clone = myself.clone();
                tokio::spawn(async move {
                    ractor::concurrency::sleep(Duration::from_secs(20)).await;
                    myself_clone
                        .cast(CityUpdaterMessage::PrintCityJson)
                        .expect("Failed to schedule next print city.json");
                });
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
    #[instrument(
        skip(self),
        fields(
            cities_file = %self.cities_file_path,
            operation.status = tracing::field::Empty,
            cities.initial_count = tracing::field::Empty,
            cities.final_count = tracing::field::Empty,
            api.countries_fetched = tracing::field::Empty,
            processing.duration_ms = tracing::field::Empty
        )
    )]
    async fn update_cities_file(
        &self,
    ) -> Result<UpdateStats, Box<dyn std::error::Error + Send + Sync>> {
        let operation_start = std::time::Instant::now();
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

                        let city_entry = existing_cities
                            .get(&city.city)
                            .map(|existing| CityEntry {
                                city_code: existing.city_code.clone(),
                                city_name: city.city.clone(),
                                country_name: country.name.clone(),
                                country_code: country.code.clone(),
                                image_url: existing.image_url.clone(),
                                latitude: if existing.latitude != 0.0 {
                                    existing.latitude
                                } else {
                                    0.0
                                },
                                longitude: if existing.longitude != 0.0 {
                                    existing.longitude
                                } else {
                                    0.0
                                },
                            })
                            .unwrap_or_else(|| {
                                stats.new_cities_added += 1;
                                CityEntry {
                                    city_code: String::new(),
                                    city_name: city.city.clone(),
                                    country_name: country.name.clone(),
                                    country_code: country.code.clone(),
                                    image_url: String::new(),
                                    latitude: 0.0,
                                    longitude: 0.0,
                                }
                            });

                        existing_cities.insert(city.city, city_entry);
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
        let final_count = existing_cities.len();
        self.save_cities(existing_cities).await?;

        let processing_duration = operation_start.elapsed();

        // Record metrics in the span for observability
        let current_span = tracing::Span::current();
        current_span.record("operation.status", "success");
        current_span.record("cities.initial_count", initial_count);
        current_span.record("cities.final_count", final_count);
        current_span.record("api.countries_fetched", stats.countries_processed);
        current_span.record(
            "processing.duration_ms",
            processing_duration.as_millis() as u64,
        );

        // Set OpenTelemetry span status to OK (when available)
        // Note: This requires tracing_opentelemetry feature to be enabled
        #[cfg(all(feature = "tracing-opentelemetry", feature = "debug_log"))]
        {
            use tracing_opentelemetry::OpenTelemetrySpanExt;
            current_span.set_status(opentelemetry::trace::Status::Ok);
        }

        debug!(
            initial_count = initial_count,
            final_count = final_count,
            new_cities_added = stats.new_cities_added,
            processing_duration_ms = processing_duration.as_millis(),
            "City update operation completed successfully"
        );

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
            city_map.insert(city.city_name.clone(), city);
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
