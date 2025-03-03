mod inhibitor;
mod poke;

use std::time::Duration;

use crate::Inhibitor;

pub use self::inhibitor::WindowsInhibitor;

pub(crate) struct InnerInhibitor(#[allow(dead_code)] WindowsInhibitor);

impl Inhibitor {
    pub fn new(_program: &str, _reason: &str, frequency: Duration) -> anyhow::Result<Self> {
        let inner = InnerInhibitor(WindowsInhibitor::new(frequency)?);

        Ok(Self { _inner: inner })
    }
}
