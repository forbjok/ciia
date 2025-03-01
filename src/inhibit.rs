use std::{cell::LazyCell, env, time::Duration};

use anyhow::Context;
use tracing::{debug, error, info};

const DBUS_TIMEOUT: Duration = Duration::from_secs(10);

const FREEDESKTOP_DEST: &str = "org.freedesktop.ScreenSaver";
const FREEDESKTOP_PATH: &str = "/org/freedesktop/ScreenSaver";
const FREEDESKTOP_INHIBIT_MEMBER: &str = "Inhibit";
const FREEDESKTOP_UNINHIBIT_MEMBER: &str = "UnInhibit";

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        debug!("Connecting to D-Bus.");
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });
}

pub trait Inhibitable {
    fn inhibit(&mut self) -> anyhow::Result<()>;
    fn uninhibit(&mut self) -> anyhow::Result<()>;
}

pub enum Inhibitor {
    Command(Vec<&'static str>),
    FreeDesktop { cookie: Option<u32> },
}

impl Inhibitor {
    pub fn new_freedesktop() -> Self {
        Self::FreeDesktop { cookie: None }
    }
}

impl Inhibitable for Inhibitor {
    fn inhibit(&mut self) -> anyhow::Result<()> {
        match self {
            Inhibitor::Command(cmd) => {
                let (program, args) = cmd.split_first().context("Couldn't split command")?;

                std::process::Command::new(program)
                    .args(args)
                    .status()
                    .context("Error executing command inhibitor")?;
            }
            Inhibitor::FreeDesktop { cookie } => {
                if cookie.is_some() {
                    return Ok(());
                }

                debug!("Inhibiting FreeDesktop.");

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(FREEDESKTOP_DEST, FREEDESKTOP_PATH, DBUS_TIMEOUT);

                    let (new_cookie,): (u32,) = proxy
                        .method_call(
                            FREEDESKTOP_DEST,
                            FREEDESKTOP_INHIBIT_MEMBER,
                            ("CIIA", "Controller input detected"),
                        )
                        .context("Error inhibiting FreeDesktop")?;

                    *cookie = Some(new_cookie);

                    Ok(())
                })?;
            }
        }

        Ok(())
    }

    fn uninhibit(&mut self) -> anyhow::Result<()> {
        match self {
            Inhibitor::Command(_) => {}
            Inhibitor::FreeDesktop { cookie } => {
                let Some(cookie) = cookie.take() else {
                    return Ok(());
                };

                debug!("Uninhibiting FreeDesktop.");

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(FREEDESKTOP_DEST, FREEDESKTOP_PATH, DBUS_TIMEOUT);

                    let () = proxy
                        .method_call(FREEDESKTOP_DEST, FREEDESKTOP_UNINHIBIT_MEMBER, (cookie,))
                        .context("Error uninhibiting FreeDesktop")?;

                    Ok(())
                })?;
            }
        }

        Ok(())
    }
}

impl<T> Inhibitable for Vec<T>
where
    T: Inhibitable,
{
    fn inhibit(&mut self) -> anyhow::Result<()> {
        info!("Inhibit!");

        for inhibitor in self.iter_mut() {
            if let Err(err) = inhibitor.inhibit() {
                error!("{}", err);
            }
        }

        Ok(())
    }

    fn uninhibit(&mut self) -> anyhow::Result<()> {
        for inhibitor in self.iter_mut() {
            if let Err(err) = inhibitor.uninhibit() {
                error!("{}", err);
            }
        }

        Ok(())
    }
}

pub fn autodetect_inhibitors() -> anyhow::Result<Vec<Inhibitor>> {
    enum Desktop {
        Kde,
        Gnome,
        Xfce,
        Cinnamon,
    }

    let desktop = if let Ok(xcd) = env::var("XDG_CURRENT_DESKTOP") {
        match xcd.as_str() {
            "KDE" => {
                info!("KDE desktop environment detected.");
                Some(Desktop::Kde)
            }
            "GNOME" => {
                info!("GNOME desktop environment detected.");
                Some(Desktop::Gnome)
            }
            "XFCE" => {
                info!("XFCE desktop environment detected.");
                Some(Desktop::Xfce)
            }
            "X-Cinnamon" => {
                info!("Cinnamon desktop environment detected.");
                Some(Desktop::Cinnamon)
            }
            _ => None,
        }
    } else {
        None
    };

    if desktop.is_none() {
        info!("No desktop environment detected. Using FreeDesktop inhibitor.");
    }

    let mut inhibitors = Vec::new();

    match desktop {
        None | Some(Desktop::Kde | Desktop::Gnome | Desktop::Cinnamon) => {
            inhibitors.push(Inhibitor::new_freedesktop());
        }
        Some(Desktop::Xfce) => {
            inhibitors.push(Inhibitor::Command(vec!["xset", "s", "reset"]));
        }
    }

    Ok(inhibitors)
}
