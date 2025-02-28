use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime},
};

use gilrs::{Event, EventType, Gilrs};
use tracing::{debug, info, trace};

use crate::{config::Config, wake};

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
    let wakers = wake::autodetect_wakers()?;

    let interval = Duration::from_secs(config.interval);
    let wake_cooldown = Duration::from_secs(config.wake_cooldown);

    let wake = move || {
        info!("Wake!");

        for waker in wakers.iter() {
            _ = waker.execute();
        }
    };

    let join_handle = std::thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        // Did we get an event since the last wake?
        let mut user_activity_detected = false;

        // Time when the last wake was performed
        let mut last_wake_at: SystemTime = SystemTime::now();

        wake();

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

            if user_activity_detected {
                let now = SystemTime::now();

                if let Ok(since) = now.duration_since(last_wake_at) {
                    if since > wake_cooldown {
                        wake();

                        user_activity_detected = false;
                        last_wake_at = now;
                    }
                }
            }

            std::thread::sleep(interval);
        }
    });

    Ok(Engine { join_handle })
}
