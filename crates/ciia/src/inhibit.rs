use std::env;

use anyhow::Context;
use idle_inhibit::FreeDesktopInhibitor;
use tracing::{error, info};

pub trait Inhibitable {
    fn inhibit(&mut self) -> anyhow::Result<()>;
    fn uninhibit(&mut self) -> anyhow::Result<()>;
}

pub enum Inhibitor {
    Command(Vec<&'static str>),
    FreeDesktop(Option<FreeDesktopInhibitor>),
}

impl Inhibitor {
    pub fn new_freedesktop() -> Self {
        Self::FreeDesktop(None)
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
            Inhibitor::FreeDesktop(ih) => {
                if ih.is_some() {
                    return Ok(());
                }

                *ih = Some(FreeDesktopInhibitor::new("CIIA", "Controller input detected")?);
            }
        }

        Ok(())
    }

    fn uninhibit(&mut self) -> anyhow::Result<()> {
        match self {
            Inhibitor::Command(_) => {}
            Inhibitor::FreeDesktop(ih) => {
                *ih = None;
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
