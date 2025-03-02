use std::{cell::LazyCell, time::Duration};

use anyhow::Context;
use tracing::{debug, error};

const DBUS_TIMEOUT: Duration = Duration::from_secs(10);

const DEST: &str = "org.freedesktop.ScreenSaver";
const PATH: &str = "/org/freedesktop/ScreenSaver";
const INHIBIT_MEMBER: &str = "Inhibit";
const UNINHIBIT_MEMBER: &str = "UnInhibit";

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        debug!("Connecting to D-Bus.");
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });
}

/// Inhibitor that uses the FreeDesktop idle inhibition spec to inhibit idle behavior.
/// https://specifications.freedesktop.org/idle-inhibit-spec/latest/
///
/// Known to work with modern versions of KDE, GNOME, Cinnamon and COSMIC.
pub struct FreeDesktopInhibitor {
    cookie: Option<u32>,
}

impl FreeDesktopInhibitor {
    pub fn new(program: &str, reason: &str) -> anyhow::Result<Self> {
        debug!("Inhibiting FreeDesktop.");

        DBUS_SESSION_CONN.with(|c| -> anyhow::Result<_> {
            let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);

            let (cookie,): (u32,) = proxy
                .method_call(DEST, INHIBIT_MEMBER, (program, reason))
                .context("Error inhibiting FreeDesktop")?;

            let cookie = Some(cookie);

            Ok(Self { cookie })
        })
    }
}

impl Drop for FreeDesktopInhibitor {
    fn drop(&mut self) {
        let Some(cookie) = self.cookie.take() else {
            return;
        };

        debug!("Uninhibiting FreeDesktop.");

        let res = DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
            let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);

            let () = proxy
                .method_call(DEST, UNINHIBIT_MEMBER, (cookie,))
                .context("Error uninhibiting FreeDesktop")?;

            Ok(())
        });

        if let Err(err) = res {
            error!("{}", err);
        }
    }
}
