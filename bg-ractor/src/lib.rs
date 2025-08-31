use ractor::{concurrency::Duration, Actor, ActorProcessingErr, ActorRef};

// Re-export cities module
pub mod cities;
pub use cities::{
    City, CityApiProvider, CityEntry, CityIterator, CityUpdaterMessage, Country,
    CountryCitiesResult, PeriodicCityUpdater,
};

/// Start the cities polling background actor
///
/// # Arguments
/// * `api_provider` - Implementation of CityApiProvider trait
/// * `update_interval` - How often to update cities.json
/// * `heartbeat_interval` - How often to log heartbeat messages
/// * `cities_file_path` - Path to the cities.json file
///
/// # Returns
/// ActorRef for the spawned cities updater actor
pub async fn start_cities_polling<T>(
    api_provider: T,
    update_interval: Duration,
    heartbeat_interval: Duration,
    cities_file_path: String,
) -> Result<ActorRef<CityUpdaterMessage>, Box<dyn std::error::Error + Send + Sync>>
where
    T: CityApiProvider,
{
    let city_updater = PeriodicCityUpdater::new(
        update_interval,
        heartbeat_interval,
        cities_file_path,
        api_provider,
    );

    let (actor_ref, _handle) = Actor::spawn(Some("cities_updater".to_string()), city_updater, ())
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    Ok(actor_ref)
}

/// Start the cities polling background actor with intervals in seconds
///
/// # Arguments
/// * `api_provider` - Implementation of CityApiProvider trait
/// * `update_interval_secs` - How often to update cities.json (in seconds)
/// * `heartbeat_interval_secs` - How often to log heartbeat messages (in seconds)
/// * `cities_file_path` - Path to the cities.json file
///
/// # Returns
/// ActorRef for the spawned cities updater actor
pub async fn start_cities_polling_with_secs<T>(
    api_provider: T,
    update_interval_secs: u32,
    heartbeat_interval_secs: u32,
    cities_file_path: String,
) -> Result<ActorRef<CityUpdaterMessage>, Box<dyn std::error::Error + Send + Sync>>
where
    T: CityApiProvider,
{
    start_cities_polling(
        api_provider,
        Duration::from_secs(update_interval_secs as u64),
        Duration::from_secs(heartbeat_interval_secs as u64),
        cities_file_path,
    )
    .await
}

// Keep the original periodic actor for backward compatibility
pub struct PeriodicActor {
    interval: Duration,
    message: String,
}

impl PeriodicActor {
    pub fn new(interval: Duration, message: String) -> Self {
        Self { interval, message }
    }
}

#[derive(Clone)]
pub struct PeriodicActorState {
    counter: u64,
}

pub enum PeriodicMessage {
    Tick,
}

#[ractor::async_trait]
impl Actor for PeriodicActor {
    type Msg = PeriodicMessage;
    type State = PeriodicActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        println!("Starting periodic actor with interval: {:?}", self.interval);

        // Schedule the first tick
        myself
            .cast(PeriodicMessage::Tick)
            .expect("Failed to send initial message");

        Ok(PeriodicActorState { counter: 0 })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PeriodicMessage::Tick => {
                state.counter += 1;
                println!("[{}] {}", state.counter, self.message);

                // Sleep for the interval, then schedule the next tick
                ractor::concurrency::sleep(self.interval).await;
                myself
                    .cast(PeriodicMessage::Tick)
                    .expect("Failed to send message to self");
            }
        }
        Ok(())
    }
}
