use anyhow::Context;

use crate::poke::PokeImpl;

pub trait CommandPokeImpl: PokeImpl {
    const PROGRAM: &str;
    const ARGS: &[&str];
}

impl<T> PokeImpl for T
where
    T: CommandPokeImpl,
{
    fn poke() -> anyhow::Result<()> {
        std::process::Command::new(Self::PROGRAM)
            .args(Self::ARGS)
            .status()
            .context("Error executing command inhibitor")?;

        Ok(())
    }
}
