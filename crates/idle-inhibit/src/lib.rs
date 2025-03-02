mod freedesktop;
mod x11;

use std::sync::LazyLock;
use std::{env, time::Duration};

use tracing::info;

pub use self::freedesktop::FreeDesktopInhibitor;
pub use self::x11::X11Inhibitor;

static DESKTOP: LazyLock<Desktop> = LazyLock::new(|| {
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
            "DDE" => {
                info!("Deepin desktop environment detected.");
                Desktop::Deepin
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

#[derive(Clone, Copy, PartialEq)]
pub enum Desktop {
    Kde,
    Gnome,
    Xfce,
    Cinnamon,
    Lxde,
    LxQt,
    Mate,
    Cosmic,
    Deepin,
    Unknown,
}

#[allow(dead_code)]
enum InnerInhibitor {
    FreeDesktop(FreeDesktopInhibitor),
    X11(X11Inhibitor),
}

pub struct Inhibitor {
    _inner: InnerInhibitor,
}

impl Inhibitor {
    pub fn new(program: &str, reason: &str, _frequency: Duration) -> anyhow::Result<Self> {
        let inner = match *DESKTOP {
            Desktop::Kde
            | Desktop::Gnome
            | Desktop::Cinnamon
            | Desktop::Unknown
            | Desktop::Cosmic
            | Desktop::Deepin => InnerInhibitor::FreeDesktop(FreeDesktopInhibitor::new(program, reason)?),
            Desktop::Xfce | Desktop::Lxde | Desktop::LxQt | Desktop::Mate => InnerInhibitor::X11(X11Inhibitor::new()?),
        };

        Ok(Self { _inner: inner })
    }
}

/// Get the autodetected desktop environment.
pub fn desktop() -> Desktop {
    *DESKTOP
}
