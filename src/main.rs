mod config;
mod engine;
mod inhibit;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use config::Config;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    initialize_logging();

    // Shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));

    // Set break (Ctrl-C) handler.
    ctrlc::set_handler({
        let shutdown = shutdown.clone();

        move || {
            info!("Shutdown requested by user.");
            shutdown.store(true, Ordering::Release);
        }
    })
    .unwrap_or_else(|err| error!("Error setting Ctrl-C handler: {}", err));

    let config = config::get_default_config_path()
        .map(|p| Config::load_or_write_default_from_location(&p))
        .unwrap_or_else(|| Ok(Config::default()))?;

    engine::run(&config, shutdown)?.join();

    Ok(())
}

fn initialize_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default tracing subscriber failed!");
}
