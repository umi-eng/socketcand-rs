#![cfg_attr(not(test), no_std)]

pub mod beacon;
pub mod wire;

/// Bus network interface.
#[derive(Debug)]
pub struct Bus<'a> {
    name: &'a str,
}
