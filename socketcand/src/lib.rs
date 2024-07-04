#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

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
    /// No bus.
    NoBus,
    /// Broadcast mode.
    Broadcast,
    /// Raw mode.
    Raw,
    /// Control mode.
    Control,
    /// ISO-TP (ISO 15765-2) mode.
    IsoTp,
}
