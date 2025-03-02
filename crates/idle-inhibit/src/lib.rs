mod freedesktop;

use std::{cell::LazyCell, time::Duration};

use tracing::debug;

pub use freedesktop::FreeDesktopInhibitor;

const DBUS_TIMEOUT: Duration = Duration::from_secs(10);

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        debug!("Connecting to D-Bus.");
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });
}
