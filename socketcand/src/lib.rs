#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use core::fmt::Display;

pub mod beacon;
pub mod wire;

/// CAN network bus.
///
/// Rather than allowing arbitrary bus names and having to store strings, bus
/// names must be of the form `vcanN` or `canN` where `N` is a positive
/// integer.
///
/// # Example
/// ```rust
/// use socketcand::Bus;
///
/// let bus = Bus::new(0); // same as `can0`
/// let bus = Bus::new_virtual(7); // same as `vcan7`
/// ```
#[derive(Debug, Clone)]
pub struct Bus {
    index: usize,
    virt: bool,
}

impl Bus {
    /// Create a new [`Bus`] instance.
    pub fn new(index: usize) -> Self {
        Self { index, virt: false }
    }

    /// Create a new virtual [`Bus`] instance.
    pub fn new_virtual(index: usize) -> Self {
        Self { index, virt: true }
    }

    /// Returns the bus index number.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns if the bus is a virtual bus.
    pub fn is_virtual(&self) -> bool {
        self.virt
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.virt {
            write!(f, "vcan{}", self.index)
        } else {
            write!(f, "can{}", self.index)
        }
    }
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for Bus {
    fn format(&self, fmt: defmt::Formatter) {
        if self.virt {
            defmt::write!(fmt, "vcan{}", self.index)
        } else {
            defmt::write!(fmt, "can{}", self.index)
        }
    }
}

/// Connection mode.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Mode {
    /// Broadcast mode.
    Broadcast,
    /// Raw mode.
    Raw,
    /// Control mode.
    Control,
    /// ISO-TP (ISO 15765-2) mode.
    IsoTp,
}
