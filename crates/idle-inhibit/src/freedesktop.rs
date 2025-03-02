use anyhow::Context;
use tracing::{debug, error};

use crate::{DBUS_SESSION_CONN, DBUS_TIMEOUT};

const DEST: &str = "org.freedesktop.ScreenSaver";
const PATH: &str = "/org/freedesktop/ScreenSaver";
const INHIBIT_MEMBER: &str = "Inhibit";
const UNINHIBIT_MEMBER: &str = "UnInhibit";

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
