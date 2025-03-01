use std::{cell::LazyCell, env, time::Duration};

use anyhow::Context;
use tracing::{debug, error, info};

const DBUS_TIMEOUT: Duration = Duration::from_secs(10);

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        info!("Connecting to D-Bus.");
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });
}

pub trait Inhibitable {
    fn inhibit(&mut self) -> anyhow::Result<()>;
    fn uninhibit(&mut self) -> anyhow::Result<()>;
}

pub enum Inhibitor {
    Command(Vec<&'static str>),
    Kde,
    Gnome { cookie: Option<u32> },
}

impl Inhibitable for Inhibitor {
    fn inhibit(&mut self) -> anyhow::Result<()> {
        match self {
            Inhibitor::Command(cmd) => {
                let (program, args) = cmd.split_first().context("Couldn't split command")?;

                std::process::Command::new(program)
                    .args(args)
                    .status()
                    .inspect_err(|err| error!("Error executing command inhibitor: {}", err))?;
            }
            Inhibitor::Kde => {
                const DEST: &str = "org.freedesktop.ScreenSaver";
                const PATH: &str = "/org/freedesktop/ScreenSaver";
                const MEMBER: &str = "SimulateUserActivity";

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);
                    let () = proxy
                        .method_call(DEST, MEMBER, ())
                        .context("Error executing KDE inhibitor")?;

                    Ok(())
                })?;
            }
            Inhibitor::Gnome { cookie } => {
                const DEST: &str = "org.freedesktop.ScreenSaver";
                const PATH: &str = "/org/freedesktop/ScreenSaver";
                const INHIBIT_MEMBER: &str = "Inhibit";

                if cookie.is_some() {
                    return Ok(());
                }

                debug!("Inhibiting GNOME.");

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);

                    let (new_cookie,): (u32,) = proxy
                        .method_call(DEST, INHIBIT_MEMBER, ("ciia", "Controller input detected"))
                        .context("Error inhibiting GNOME")?;

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
            Inhibitor::Kde => {}
            Inhibitor::Gnome { cookie } => {
                const DEST: &str = "org.freedesktop.ScreenSaver";
                const PATH: &str = "/org/freedesktop/ScreenSaver";
                const UNINHIBIT_MEMBER: &str = "UnInhibit";

                let Some(cookie) = cookie.take() else {
                    return Ok(());
                };

                debug!("Uninhibiting GNOME.");

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);

                    let () = proxy
                        .method_call(DEST, UNINHIBIT_MEMBER, (cookie,))
                        .context("Error uninhibiting GNOME")?;

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
            _ = inhibitor.inhibit();
        }

        Ok(())
    }

    fn uninhibit(&mut self) -> anyhow::Result<()> {
        for inhibitor in self.iter_mut() {
            _ = inhibitor.uninhibit();
        }

        Ok(())
    }
}

pub fn autodetect_inhibitors() -> anyhow::Result<Vec<Inhibitor>> {
    let mut inhibitors = Vec::new();

    if let Ok(xcd) = env::var("XDG_CURRENT_DESKTOP") {
        match xcd.as_str() {
            "KDE" => {
                info!("KDE desktop environment detected.");

                inhibitors.push(Inhibitor::Kde);
            }
            "GNOME" => {
                info!("GNOME desktop environment detected.");

                inhibitors.push(Inhibitor::Gnome { cookie: None });
            }
            "XFCE" => {
                info!("XFCE desktop environment detected.");

                inhibitors.push(Inhibitor::Command(vec!["xset", "s", "reset"]));
            }
            "X-Cinnamon" => {
                info!("Cinnamon desktop environment detected.");

                inhibitors.push(Inhibitor::Gnome { cookie: None });
            }
            _ => {}
        }
    }

    Ok(inhibitors)
}
