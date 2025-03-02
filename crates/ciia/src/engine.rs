use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime},
};

use gilrs::{Event, EventType, Gilrs};
use tracing::{debug, info, trace};

use crate::{
    config::Config,
    inhibit::{self, Inhibitable as _},
};

/// Represents a running instance of a CIIA engine.
pub struct Engine {
    join_handle: std::thread::JoinHandle<()>,
}

impl Engine {
    /// Wait for engine thread to finish
    pub fn join(self) {
        self.join_handle.join().unwrap();
    }
}

/// Run CIIA engine
pub fn run(config: &Config, shutdown: Arc<AtomicBool>) -> anyhow::Result<Engine> {
    let mut inhibitors = inhibit::autodetect_inhibitors()?;

    let interval = Duration::from_secs(config.interval);
    let inhibit_frequency = Duration::from_secs(config.inhibit_frequency);

    let join_handle = std::thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        // Are we currently inhibiting?
        let mut is_inhibiting = false;

        // Did we detect user activity since the last inhibit?
        let mut user_activity_detected = false;

        // Time when the last inhibit was performed
        let mut last_inhibit_at: SystemTime = SystemTime::UNIX_EPOCH;

        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }

            trace!("...");

            while let Some(Event { id, event, .. }) = gilrs.next_event() {
                debug!("Event from {id}: {:?}", event);

                if !user_activity_detected
                    && matches!(
                        event,
                        EventType::AxisChanged(..)
                            | EventType::ButtonChanged(..)
                            | EventType::ButtonPressed(..)
                            | EventType::ButtonReleased(..)
                    )
                {
                    info!("User activity detected!");

                    user_activity_detected = true;
                }
            }

            let now = SystemTime::now();

            if let Ok(since) = now.duration_since(last_inhibit_at) {
                if since > inhibit_frequency {
                    if user_activity_detected {
                        _ = inhibitors.inhibit();

                        is_inhibiting = true;
                        user_activity_detected = false;
                        last_inhibit_at = now;
                    } else if is_inhibiting {
                        _ = inhibitors.uninhibit();
                        is_inhibiting = false;
                    }
                }
            }

            std::thread::sleep(interval);
        }
    });

    Ok(Engine { join_handle })
}
