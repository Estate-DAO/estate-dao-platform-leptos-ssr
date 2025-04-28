use once_cell::sync::OnceCell;
use std::env;
use tracing_subscriber::Layer;
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

#[cfg(feature = "debug_log_file")]
use tracing_appender::non_blocking::WorkerGuard;
// Static to hold the guard and keep it alive for the lifetime of the app
#[cfg(feature = "debug_log_file")]
static FILE_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

fn create_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("estate_fe=debug,tower_http=info"))
}

pub fn init_tracing() {
    // Layer for writing to terminal (stdout)
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_filter(create_env_filter());

    #[cfg(feature = "debug_log_file")]
    {
        use tracing_appender::rolling;
        // Create a rolling daily file appender under "logs/estate_fe_local.log.YYYY-MM-DD"
        let file_appender = rolling::daily("logs", "estate_fe_local.log");

        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

        // Store the guard globally to keep it alive
        let _ = FILE_GUARD.set(guard);

        // Layer for writing to file
        let file_layer = fmt::layer()
            .with_writer(file_writer)
            .with_ansi(false)
            .with_target(true)
            .with_filter(create_env_filter());

        // Build and initialize the subscriber
        Registry::default()
            .with(stdout_layer)
            .with(file_layer)
            .init();
    }

    #[cfg(not(feature = "debug_log_file"))]
    {
        // Build and initialize the subscriber
        Registry::default().with(stdout_layer).init();
    }
}
