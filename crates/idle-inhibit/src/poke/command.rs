use anyhow::Context;

use crate::poke::Poker;

pub trait CommandPoker: Poker {
    const PROGRAM: &str;
    const ARGS: &[&str];
}

impl<T> Poker for T
where
    T: CommandPoker,
{
    fn poke(&mut self) -> anyhow::Result<()> {
        std::process::Command::new(Self::PROGRAM)
            .args(Self::ARGS)
            .status()
            .context("Error executing command inhibitor")?;

        Ok(())
    }
}
