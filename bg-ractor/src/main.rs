use bg_ractor::{
    start_cities_polling, City, CityApiProvider, CityIterator, Country, CountryCitiesResult,
};
use ractor::concurrency::Duration;
use std::future::Future;
use std::pin::Pin;
use tracing::{info, instrument, Instrument};

// Mock API provider for testing bg-ractor independently
#[derive(Clone)]
pub struct MockCityApiProvider;

pub struct MockCityIterator {
    current: usize,
    total: usize,
}

impl CityIterator for MockCityIterator {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<CountryCitiesResult>> + Send + '_>> {
        Box::pin(async move {
            if self.current >= self.total {
                return None;
            }

            self.current += 1;
            let country = Country {
                code: format!("C{:02}", self.current),
                name: format!("Country {}", self.current),
            };
            let cities = vec![
                City {
                    city: format!("City A {}", self.current),
                },
                City {
                    city: format!("City B {}", self.current),
                },
            ];

            // Simulate some processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            Some(Ok((country, cities)))
        })
    }

    fn progress(&self) -> (usize, usize) {
        (self.current, self.total)
    }
}

#[async_trait::async_trait]
impl CityApiProvider for MockCityApiProvider {
    type Iterator = MockCityIterator;
    type Error = std::io::Error;

    async fn get_all_cities(&self) -> Result<Self::Iterator, Self::Error> {
        Ok(MockCityIterator {
            current: 0,
            total: 5, // Mock 5 countries
        })
    }
}

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry (same as ssr main.rs)
    use telemetry_axum;
    let config_telemetry = telemetry_axum::Config {
        exporter: telemetry_axum::Exporter::Stdout,
        propagate: false,
        ..Default::default()
    };

    let (logger_provider, tracer_provider, metrics_provider) =
        telemetry_axum::init_telemetry(&config_telemetry).map_err(|e| anyhow::Error::new(e))?;

    info!("Starting bg-ractor with telemetry");

    // Create mock API provider
    let api_provider = MockCityApiProvider;

    // Start cities polling with 3-second heartbeat interval
    let current_span = tracing::Span::current();
    let actor_ref = start_cities_polling(
        api_provider,
        Duration::from_secs(60 * 10), // Update cities every 10 minutes
        Duration::from_secs(3),       // Heartbeat every 3 seconds
        "bg_ractor_cities.json".to_string(),
    )
    .instrument(current_span)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to start cities polling: {}", e))?;

    info!("Cities polling background actor started");

    // Let it run for 2 minutes to demonstrate heartbeat and city.json printing
    tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

    info!("Shutting down bg-ractor");
    actor_ref.stop(None);

    // Cleanup telemetry providers
    if let Some(logger_provider) = logger_provider {
        if let Err(e) = logger_provider.shutdown() {
            println!("Error shutting down logger provider: {e}");
        }
    }
    if let Err(e) = tracer_provider.shutdown() {
        println!("Error shutting down tracer provider: {e}");
    }
    if let Some(metrics_provider) = metrics_provider {
        if let Err(e) = metrics_provider.shutdown() {
            println!("Error shutting down metrics provider: {e}");
        }
    }

    Ok(())
}
