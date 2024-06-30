#![cfg_attr(not(test), no_std)]

pub mod beacon;
pub mod wire;

/// Bus network interface.
#[derive(Debug)]
pub struct Bus<'a> {
    name: &'a str,
}

/// Connection mode.
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Mode {
    #[default]
    NoBus,
    Broadcast,
    Raw,
    Control,
    IsoTp,
}
