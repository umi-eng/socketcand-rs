//! Service discovery beacon.

use crate::Bus;
use std::fmt::Formatter;

/// Port used for broadcasting service discovery datagrams.
pub const PORT: u16 = 42000;

/// Format a beacon message.
pub fn format(
    fmt: &mut Formatter<'_>,
    name: &str,
    device_kind: &str,
    description: Option<&str>,
    ip: core::net::IpAddr,
    port: u16,
    busses: &'_ [Bus<'_>],
) -> core::fmt::Result {
    write!(fmt, r#"<CANBeacon name="{}" type="{}""#, name, device_kind)?;

    if let Some(description) = description {
        write!(fmt, r#" description="{}""#, description)?;
    }

    write!(fmt, ">")?;

    write!(fmt, "\t<URL>can://{}:{}</URL>", ip, port)?;

    for bus in busses {
        write!(fmt, r#"\t<Bus name="{}">"#, bus.name)?;
    }

    write!(fmt, "</CANBeacon>")
}
