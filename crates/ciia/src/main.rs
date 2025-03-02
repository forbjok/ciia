mod config;
mod engine;

use std::{
    cell::LazyCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use config::Config;
use dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    initialize_logging();

    // Try to connect to D-Bus to check whether an instance is already running in this session.
    let _dbus_session_conn = if let Ok(dbus_session_conn) = dbus::blocking::Connection::new_session() {
        if let Ok(reply) = dbus_session_conn.request_name("org.forbsoft.CIIA", false, false, true) {
            if reply == RequestNameReply::Exists {
                info!("Another instance of CIIA is already running in this session. Exiting.");
                return Ok(());
            }
        }

        Some(dbus_session_conn)
    } else {
        None
    };

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
