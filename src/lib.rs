// #![cfg_attr(not(test), no_std)]

use core::fmt::Formatter;

pub mod parse;

#[derive(Debug)]
pub struct Frame {}

#[derive(Debug)]
enum Mode {
    NoBus,
    BroadcastManager,
    Raw,
    Control,
    IsoTp,
}

#[derive(Debug)]
pub enum DeviceType {
    SocketCan,
    Embedded,
    Adapter,
}

impl DeviceType {
    fn as_str(&self) -> &str {
        match self {
            Self::SocketCan => "SocketCAN",
            Self::Embedded => "embedded",
            Self::Adapter => "adapter",
        }
    }
}

#[derive(Debug, Default)]
pub struct Bus {}

/// Socketcand server.
#[derive(Debug)]
struct Server<'b> {
    busses: &'b [Bus],
}

impl<'b> Server<'b> {
    pub fn new(busses: &'b [Bus]) -> Self {
        Self { busses }
    }

    /// Format output to be transmitted over the beacon socket for discovery.
    pub fn beacon_output(
        &self,
        fmt: &mut Formatter<'_>,
        name: &str,
        device_type: DeviceType,
        description: Option<&str>,
        ip: core::net::IpAddr,
        port: u16,
    ) -> core::fmt::Result {
        write!(
            fmt,
            r#"<CANBeacon name="{}" type="{}""#,
            name,
            device_type.as_str()
        )?;

        if let Some(description) = description {
            write!(fmt, r#" description="{}""#, description)?;
        }

        write!(fmt, ">")?;

        write!(fmt, "\t<URL>can://{}:{}</URL>", ip, port)?;

        for (index, _) in self.busses.iter().enumerate() {
            write!(fmt, r#"\t<Bus name="can{}">"#, index)?;
        }

        write!(fmt, "</CANBeacon>")?;

        Ok(())
    }
}
