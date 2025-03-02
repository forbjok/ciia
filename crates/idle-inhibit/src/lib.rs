mod command;
mod freedesktop;
mod poke;
mod xset;

use std::{cell::LazyCell, env, time::Duration};

use tracing::{debug, info};
use xset::XsetInhibitor;

pub use self::freedesktop::FreeDesktopInhibitor;
pub use self::poke::PokeInhibitor;

const DBUS_TIMEOUT: Duration = Duration::from_secs(10);

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        debug!("Connecting to D-Bus.");
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });

    static DESKTOP: LazyCell<Desktop> = LazyCell::new(|| {
        let desktop = if let Ok(xcd) = env::var("XDG_CURRENT_DESKTOP") {
            match xcd.as_str() {
                "KDE" => {
                    info!("KDE desktop environment detected.");
                    Desktop::Kde
                }
                "GNOME" => {
                    info!("GNOME desktop environment detected.");
                    Desktop::Gnome
                }
                "XFCE" => {
                    info!("XFCE desktop environment detected.");
                    Desktop::Xfce
                }
                "X-Cinnamon" => {
                    info!("Cinnamon desktop environment detected.");
                    Desktop::Cinnamon
                }
                "LXDE" => {
                    info!("LXDE desktop environment detected.");
                    Desktop::Lxde
                }
                "LXQt" => {
                    info!("LXQt desktop environment detected.");
                    Desktop::LxQt
                }
                "MATE" => {
                    info!("MATE desktop environment detected.");
                    Desktop::Mate
                }
                "COSMIC" => {
                    info!("COSMIC desktop environment detected.");
                    Desktop::Cosmic
                }
                _ => Desktop::Unknown,
            }
        } else {
            Desktop::Unknown
        };

        if desktop == Desktop::Unknown {
            info!("No desktop environment detected. Using FreeDesktop inhibitor.");
        }

        desktop
    });
}

#[derive(Clone, Copy, PartialEq)]
enum Desktop {
    Kde,
    Gnome,
    Xfce,
    Cinnamon,
    Lxde,
    LxQt,
    Mate,
    Cosmic,
    Unknown,
}

#[allow(dead_code)]
enum InnerInhibitor {
    FreeDesktop(FreeDesktopInhibitor),
    Xset(XsetInhibitor),
}

pub struct Inhibitor {
    _inner: InnerInhibitor,
}

impl Inhibitor {
    pub fn new(program: &str, reason: &str, frequency: Duration) -> anyhow::Result<Self> {
        let desktop = DESKTOP.with(|d| **d);

        let inner = match desktop {
            Desktop::Kde | Desktop::Gnome | Desktop::Cinnamon | Desktop::Unknown | Desktop::Cosmic => {
                InnerInhibitor::FreeDesktop(FreeDesktopInhibitor::new(program, reason)?)
            }
            Desktop::Xfce | Desktop::Lxde | Desktop::LxQt | Desktop::Mate => {
                InnerInhibitor::Xset(XsetInhibitor::new(frequency)?)
            }
        };

        Ok(Self { _inner: inner })
    }
}
