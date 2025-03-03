#[cfg(windows)]
mod app;
mod config;
mod engine;
mod session;

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

    let Some(_instance) = session::session()? else {
        info!("Another instance of CIIA is already running in this session. Exiting.");
        return Ok(());
    };

    let _app = app::main();

    let mut gilrs = gilrs::Gilrs::new().unwrap();
    /*
    let mut gilrs = {
        let b = gilrs::GilrsBuilder::new()
        .with_default_filters(false)
        .set_update_state(false);

        b.build().unwrap()
    }; */
    loop {
        //dbg!(".");
        while let Some(ev) =
            gilrs::Filter::filter_ev(&gilrs.next_event(), &gilrs::ev::filter::axis_dpad_to_button, &mut gilrs)
        {
            dbg!("EventX", &ev);
            gilrs.update(&ev);
        }
    }

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
