use std::time::Duration;

use windows::Win32::System::Power::{ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED};

use super::poke::{PokeInhibitor, Poker};

struct WindowsPoker;

pub struct WindowsInhibitor(#[allow(dead_code)] PokeInhibitor<WindowsPoker>);

impl Poker for WindowsPoker {
    fn poke(&mut self) -> anyhow::Result<()> {
        unsafe {
            windows::Win32::System::Power::SetThreadExecutionState(
                ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED | ES_CONTINUOUS,
            );
        }

        Ok(())
    }
}

impl WindowsInhibitor {
    pub fn new(frequency: Duration) -> anyhow::Result<Self> {
        let inner = PokeInhibitor::new(WindowsPoker, frequency)?;

        Ok(Self(inner))
    }
}
