use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use gilrs::{Event, EventType, Gilrs};
use idle_inhibit::Inhibitor;
use tracing::{debug, error, info, trace};

use crate::config::Config;

/// Represents a running instance of a CIIA engine.
pub struct Engine {
    join_handle: thread::JoinHandle<()>,
}

impl Engine {
    /// Wait for engine thread to finish
    pub fn join(self) {
        self.join_handle.join().unwrap();
    }
}

/// Run CIIA engine
pub fn run(config: &Config, shutdown: Arc<AtomicBool>) -> anyhow::Result<Engine> {
    let interval = Duration::from_secs(config.interval);
    let inhibit_frequency = Duration::from_secs(config.inhibit_frequency);

    #[cfg(unix)]
    {
        // Force immediate desktop environment detection
        _ = idle_inhibit::unix::desktop();
    }

    let join_handle = thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        // Are we currently inhibiting?
        let mut inhibitor: Option<Inhibitor> = None;

        // Did we detect user activity since the last inhibit?
        let mut user_activity_detected = false;

        // Time when the last check was performed
        let mut last_check_at: Option<Instant> = None;

        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }

            trace!("...");
            thread::sleep(interval);

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

            let is_inhibiting = inhibitor.is_some();

            let now = Instant::now();

            let time_since_last = last_check_at.map(|v| now - v).unwrap_or_else(|| Duration::MAX);
            if time_since_last < inhibit_frequency && is_inhibiting {
                continue;
            }

            if user_activity_detected && !is_inhibiting {
                info!("Inhibit!");

                let res = Inhibitor::new("CIIA", "Controller input detected", inhibit_frequency);
                match res {
                    Ok(v) => inhibitor = Some(v),
                    Err(err) => error!("{}", err),
                }
            } else if !user_activity_detected && is_inhibiting {
                info!("Uninhibit!");

                inhibitor = None;
            }

            user_activity_detected = false;
            last_check_at = Some(now);
        }
    });

    Ok(Engine { join_handle })
}
