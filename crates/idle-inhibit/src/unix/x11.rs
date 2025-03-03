use std::sync::LazyLock;

use tracing::{debug, error};
use x11rb::{
    connection::Connection,
    protocol::screensaver::ConnectionExt,
    rust_connection::{DefaultStream, RustConnection},
};

static X11RB_CONN: LazyLock<RustConnection<DefaultStream>> = LazyLock::new(|| {
    debug!("Connecting to X11.");
    let (conn, _) = x11rb::connect(None).expect("Error connecting to X11");

    conn
});

/// Inhibitor that uses X11 screensaver suspend.
///
/// Known to work with modern versions of XFCE, LXDE, LXQt and MATE.
pub struct X11Inhibitor;

impl X11Inhibitor {
    pub fn new() -> anyhow::Result<Self> {
        debug!("Inhibiting X11.");

        let c = &*X11RB_CONN;
        c.screensaver_suspend(1)?;
        c.flush()?;

        Ok(Self)
    }
}

impl Drop for X11Inhibitor {
    fn drop(&mut self) {
        debug!("Uninhibiting X11.");

        let res = (|| -> anyhow::Result<()> {
            let c = &*X11RB_CONN;
            c.screensaver_suspend(0)?;
            c.flush()?;

            Ok(())
        })();

        if let Err(err) = res {
            error!("{}", err);
        }
    }
}
