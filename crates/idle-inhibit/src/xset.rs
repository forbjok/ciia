use crate::{PokeInhibitor, command::CommandPokeImpl};

pub struct XsetPokeImpl;

impl CommandPokeImpl for XsetPokeImpl {
    const PROGRAM: &str = "xset";
    const ARGS: &[&str] = &["s", "reset"];
}

pub type XsetInhibitor = PokeInhibitor<XsetPokeImpl>;
