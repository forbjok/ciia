use std::time::Duration;

use tracing::debug;

use crate::{PokeInhibitor, poke::command::CommandPoker};

pub struct XsetPoker;

impl CommandPoker for XsetPoker {
    const PROGRAM: &str = "xset";
    const ARGS: &[&str] = &["s", "reset"];
}

/// A poker inhibitor that executes `xset s reset` to keep the system from becoming idle.
/// Works for a number of X11 desktop environments.
pub struct XsetInhibitor(#[allow(dead_code)] PokeInhibitor<XsetPoker>);

impl XsetInhibitor {
    #[allow(dead_code)]
    pub fn new(frequency: Duration) -> anyhow::Result<Self> {
        debug!("Inhibiting X11 using xset.");

        Ok(Self(PokeInhibitor::<XsetPoker>::new(XsetPoker, frequency)?))
    }
}

impl Drop for XsetInhibitor {
    fn drop(&mut self) {
        debug!("Uninhibiting X11 using xset.");
    }
}
