use std::cell::LazyCell;

use tracing::{debug, error};
use x11rb::{
    connection::Connection,
    protocol::screensaver::ConnectionExt,
    rust_connection::{DefaultStream, RustConnection},
};

thread_local! {
    static X11RB_CONN: LazyCell<RustConnection<DefaultStream>> = LazyCell::new(|| {
        debug!("Connecting to X11.");
        let (conn, _) = x11rb::connect(None).expect("Error connecting to X11");

        conn
    });
}

/// Inhibitor that uses X11 screensaver suspend.
///
/// Known to work with modern versions of XFCE, LXDE, LXQt and MATE.
pub struct X11Inhibitor;

impl X11Inhibitor {
    pub fn new() -> anyhow::Result<Self> {
        debug!("Inhibiting X11.");

        X11RB_CONN.with(|c| -> anyhow::Result<_> {
            c.screensaver_suspend(1)?;
            c.flush()?;

            Ok(Self)
        })
    }
}

impl Drop for X11Inhibitor {
    fn drop(&mut self) {
        debug!("Uninhibiting X11.");

        let res = X11RB_CONN.with(|c| -> anyhow::Result<_> {
            c.screensaver_suspend(0)?;
            c.flush()?;

            Ok(())
        });

        if let Err(err) = res {
            error!("{}", err);
        }
    }
}
