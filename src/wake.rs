use std::{cell::LazyCell, env, time::Duration};

use anyhow::Context;
use tracing::{error, info};

const DBUS_TIMEOUT: Duration = Duration::from_secs(10);

thread_local! {
    static DBUS_SESSION_CONN: LazyCell<dbus::blocking::Connection> = LazyCell::new(|| {
        info!("Connecting to D-Bus.");
        dbus::blocking::Connection::new_session().expect("Error connecting to D-Bus")
    });
}

pub enum Waker {
    Command(Vec<&'static str>),
    Kde,
    Gnome,
}

impl Waker {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Waker::Command(cmd) => {
                let (program, args) = cmd.split_first().context("Couldn't split command")?;

                std::process::Command::new(program)
                    .args(args)
                    .status()
                    .inspect_err(|err| error!("Error executing command waker: {}", err))?;
            }
            Waker::Kde => {
                const DEST: &str = "org.freedesktop.ScreenSaver";
                const PATH: &str = "/org/freedesktop/ScreenSaver";
                const MEMBER: &str = "SimulateUserActivity";

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);
                    let () = proxy
                        .method_call(DEST, MEMBER, ())
                        .inspect_err(|err| error!("Error executing KDE waker: {}", err))?;

                    Ok(())
                })?;
            }
            Waker::Gnome => {
                const DEST: &str = "org.freedesktop.ScreenSaver";
                const PATH: &str = "/org/freedesktop/ScreenSaver";
                const MEMBER: &str = "Inhibit";

                DBUS_SESSION_CONN.with(|c| -> anyhow::Result<()> {
                    let proxy = c.with_proxy(DEST, PATH, DBUS_TIMEOUT);
                    let () = proxy
                        .method_call(DEST, MEMBER, ("ciia", "Controller input detected"))
                        .inspect_err(|err| error!("Error executing GNOME waker: {}", err))?;

                    Ok(())
                })?;
            }
        }

        Ok(())
    }
}

pub fn autodetect_wakers() -> anyhow::Result<Vec<Waker>> {
    let mut wakers = Vec::new();

    if let Ok(xcd) = env::var("XDG_CURRENT_DESKTOP") {
        match xcd.as_str() {
            "KDE" => {
                info!("KDE desktop environment detected.");

                wakers.push(Waker::Kde);
            }
            "GNOME" => {
                info!("GNOME desktop environment detected.");

                wakers.push(Waker::Gnome);
            }
            "XFCE" => {
                info!("XFCE desktop environment detected.");

                wakers.push(Waker::Command(vec!["xset", "s", "reset"]));
            }
            "X-Cinnamon" => {
                info!("Cinnamon desktop environment detected.");

                wakers.push(Waker::Gnome);
            }
            _ => {}
        }
    }

    Ok(wakers)
}
