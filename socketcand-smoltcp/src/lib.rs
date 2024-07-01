#![cfg_attr(not(test), no_std)]

mod server;

pub use server::Server;

/// Socketcand port.
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Port(pub u16);

impl Default for Port {
    fn default() -> Self {
        Port(29536)
    }
}
