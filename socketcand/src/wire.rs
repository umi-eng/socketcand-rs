//! Wire protocol parsing.
use core::str::FromStr;
use core::time::Duration;
use embedded_can::{ExtendedId, Id, StandardId};
use heapless::Vec;
use nom::{
    branch::alt,
    bytes::{complete::tag, streaming::take_while},
    character::{
        complete::char,
        streaming::{digit1, hex_digit1},
    },
    combinator::{map, map_res, peek},
    sequence::{delimited, terminated, tuple},
    IResult,
};

/// Parse CAN id.
fn id(input: &str) -> IResult<&str, Id> {
    let (input, (extended, id)) = tuple((
        peek(map_res(hex_digit1, |id: &str| match id.len() {
            8 => Ok(true),
            3 => Ok(false),
            _ => Err("Id length incorrect."),
        })),
        terminated(
            map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
            char(' '),
        ),
    ))(input)?;

    let id = if extended {
        match ExtendedId::new(id) {
            Some(id) => Id::Extended(id),
            None => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::MapRes,
                )))
            }
        }
    } else {
        match StandardId::new(id as u16) {
            Some(id) => Id::Standard(id),
            None => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::MapRes,
                )))
            }
        }
    };

    Ok((input, id))
}

/// Open command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Open {
    /// Interface index.
    pub index: u8,
    /// Virtual interface (e.g. `vcan0`).
    pub virt: bool,
}

fn open<'a>(input: &'a str) -> IResult<&'a str, Open> {
    let (input, (interface_type, index)) = delimited(
        tag("< open "),
        tuple((
            alt((tag("can"), tag("vcan"))),
            map_res(digit1, u8::from_str),
        )),
        tag(" >"),
    )(input)?;

    let virt = interface_type == "vcan";

    Ok((input, Open { index, virt }))
}

/// Frame job add command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Add {
    /// Interval.
    pub interval: Duration,
    /// CAN identifier.
    pub id: Id,
    /// CAN data length code.
    pub dlc: u8,
    /// CAN data.
    pub data: Vec<u8, 8>,
}

fn add<'a>(input: &'a str) -> IResult<&'a str, Add> {
    let (input, (secs, micros, id, dlc, data)) = delimited(
        tag("< add "),
        tuple((
            terminated(map_res(digit1, u64::from_str), char(' ')),
            terminated(map_res(digit1, u64::from_str), char(' ')),
            id,
            terminated(map_res(digit1, u8::from_str), char(' ')),
            map(
                take_while(|c: char| c.is_digit(16) || c == ' '),
                |bytes: &str| {
                    bytes
                        .split_whitespace()
                        .filter_map(|b| u8::from_str_radix(b, 16).ok())
                        .collect::<Vec<u8, 8>>()
                },
            ),
        )),
        char('>'),
    )(input)?;

    let interval = Duration::from_secs(secs) + Duration::from_micros(micros);

    Ok((
        input,
        Add {
            interval,
            id,
            dlc,
            data,
        },
    ))
}

/// Frame job update command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Update {
    /// CAN identifier.
    pub id: Id,
    /// CAN data length code.
    pub dlc: u8,
    /// CAN data.
    pub data: Vec<u8, 8>,
}

fn update<'a>(input: &'a str) -> IResult<&'a str, Update> {
    let (input, (id, dlc, data)) = delimited(
        tag("< update "),
        tuple((
            id,
            terminated(map_res(digit1, u8::from_str), char(' ')),
            map(
                take_while(|c: char| c.is_digit(16) || c == ' '),
                |bytes: &str| {
                    bytes
                        .split_whitespace()
                        .filter_map(|b| u8::from_str_radix(b, 16).ok())
                        .collect::<Vec<u8, 8>>()
                },
            ),
        )),
        char('>'),
    )(input)?;

    Ok((input, Update { id, dlc, data }))
}

/// Frame job delete command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Delete {
    /// CAN identifier.
    pub id: Id,
}

fn delete<'a>(input: &'a str) -> IResult<&'a str, Delete> {
    let (input, id) = delimited(tag("< delete "), id, char('>'))(input)?;

    Ok((input, Delete { id }))
}

/// Single frame send command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Send {
    /// CAN identifier.
    pub id: Id,
    /// CAN data length code.
    pub dlc: u8,
    /// CAN data.
    pub data: Vec<u8, 8>,
}

fn send<'a>(input: &'a str) -> IResult<&'a str, Send> {
    let (input, (id, dlc, data)) = delimited(
        tag("< send "),
        tuple((
            id,
            terminated(map_res(digit1, u8::from_str), char(' ')),
            map(
                take_while(|c: char| c.is_digit(16) || c == ' '),
                |bytes: &str| {
                    if bytes.trim().is_empty() {
                        Vec::new()
                    } else {
                        bytes
                            .split_whitespace()
                            .filter_map(|b| u8::from_str_radix(b, 16).ok())
                            .collect::<Vec<u8, 8>>()
                    }
                },
            ),
        )),
        char('>'),
    )(input)?;

    Ok((input, Send { id, dlc, data }))
}

/// Content filter command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Filter {
    /// Update rate.
    pub interval: Duration,
    /// CAN identifier.
    pub id: Id,
    /// CAN data length code.
    pub dlc: u8,
    /// CAN data.
    pub data: Vec<u8, 8>,
}

fn filter<'a>(input: &'a str) -> IResult<&'a str, Filter> {
    let (input, (secs, micros, id, dlc, data)) = delimited(
        tag("< filter "),
        tuple((
            terminated(map_res(digit1, u64::from_str), char(' ')),
            terminated(map_res(digit1, u64::from_str), char(' ')),
            id,
            terminated(map_res(digit1, u8::from_str), char(' ')),
            map(
                take_while(|c: char| c.is_digit(16) || c == ' '),
                |bytes: &str| {
                    bytes
                        .split_whitespace()
                        .filter_map(|b| u8::from_str_radix(b, 16).ok())
                        .collect::<Vec<u8, 8>>()
                },
            ),
        )),
        char('>'),
    )(input)?;

    let interval = Duration::from_secs(secs) + Duration::from_micros(micros);

    Ok((
        input,
        Filter {
            interval,
            id,
            dlc,
            data,
        },
    ))
}

/// Echo command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Echo;

fn echo<'a>(input: &'a str) -> IResult<&'a str, Echo> {
    let (input, _) = tag("< echo >")(input)?;

    Ok((input, Echo))
}

/// Enter raw mode command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct RawMode;

fn raw_mode<'a>(input: &'a str) -> IResult<&'a str, RawMode> {
    let (input, _) = tag("< rawmode >")(input)?;

    Ok((input, RawMode))
}

/// Enter broadcast mode command.
///
/// Broadcase mode is the default mode.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct BroadcastMode;

fn broadcast_mode<'a>(input: &'a str) -> IResult<&'a str, BroadcastMode> {
    let (input, _) = tag("< bcmode >")(input)?;

    Ok((input, BroadcastMode))
}

/// Enter control mode command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct ControlMode;

fn control_mode<'a>(input: &'a str) -> IResult<&'a str, ControlMode> {
    let (input, _) = tag("< controlmode >")(input)?;

    Ok((input, ControlMode))
}

/// Statistics setting command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Statistics {
    /// Send rate milliseconds.
    pub interval: Duration,
}

fn statistics<'a>(input: &'a str) -> IResult<&'a str, Statistics> {
    let (input, millis) = delimited(
        tag("< statistics "),
        terminated(map_res(digit1, |v: &str| u64::from_str(v)), char(' ')),
        char('>'),
    )(input)?;

    let interval = Duration::from_millis(millis);

    Ok((input, Statistics { interval }))
}

/// Command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Command {
    /// Open command.
    Open(Open),
    /// Add command.
    Add(Add),
    /// Update command.
    Update(Update),
    /// Delete command.
    Delete(Delete),
    /// Send command.
    Send(Send),
    /// Filter command.
    Filter(Filter),
    /// Echo command.
    Echo(Echo),
    /// Raw mode command.
    RawMode(RawMode),
    /// Broadcast mode command.
    BroadcastMode(BroadcastMode),
    /// Control mode command.
    ControlMode(ControlMode),
    /// Statistics command.
    Statistics(Statistics),
}

/// Parse a socketcand command.
pub fn command<'a>(input: &'a str) -> IResult<&'a str, Command> {
    alt((
        map(open, Command::Open),
        map(add, Command::Add),
        map(update, Command::Update),
        map(delete, Command::Delete),
        map(send, Command::Send),
        map(filter, Command::Filter),
        map(echo, Command::Echo),
        map(raw_mode, Command::RawMode),
        map(broadcast_mode, Command::BroadcastMode),
        map(control_mode, Command::ControlMode),
        map(statistics, Command::Statistics),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_open() {
        let (_, result) = command("< open vcan5 >").unwrap();
        assert_eq!(
            result,
            Command::Open(Open {
                index: 5,
                virt: true
            })
        );
    }

    #[test]
    fn parse_add() {
        let (_, result) =
            command("< add 1 0 123 8 11 22 33 44 55 66 77 88 >").unwrap();
        assert_eq!(
            result,
            Command::Add(Add {
                interval: Duration::from_secs(1),
                id: Id::Standard(StandardId::new(0x123).unwrap()),
                dlc: 8,
                data: Vec::from_slice(&[
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88
                ])
                .unwrap(),
            })
        );
    }

    #[test]
    fn parse_update() {
        let (_, result) = command("< update 123 3 11 22 33 >").unwrap();
        assert_eq!(
            result,
            Command::Update(Update {
                id: Id::Standard(StandardId::new(0x123).unwrap()),
                dlc: 3,
                data: Vec::from_slice(&[0x11, 0x22, 0x33,]).unwrap(),
            })
        );
    }

    #[test]
    fn parse_delete() {
        let (_, result) = command("< delete 123 >").unwrap();
        assert_eq!(
            result,
            Command::Delete(Delete {
                id: Id::Standard(StandardId::new(0x123).unwrap())
            })
        );
    }

    #[test]
    fn parse_send_no_data() {
        let (_, result) = command("< send 123 0 >").unwrap();
        assert_eq!(
            result,
            Command::Send(Send {
                id: Id::Standard(StandardId::new(0x123).unwrap()),
                dlc: 0,
                data: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_send_with_data() {
        let (_, result) = command("< send 1AAAAAAA 2 1 f1 >").unwrap();
        assert_eq!(
            result,
            Command::Send(Send {
                id: Id::Extended(ExtendedId::new(0x1AAAAAAA).unwrap()),
                dlc: 2,
                data: Vec::from_slice(&[0x1, 0xf1]).unwrap(),
            })
        );
    }

    #[test]
    fn parse_send_id_length_incorrect() {
        let result = command("< send 1234 0 >");
        assert!(result.is_err());
    }

    #[test]
    fn parse_filter() {
        let (_, result) = command("< filter 0 0 123 1 FF >").unwrap();
        assert_eq!(
            result,
            Command::Filter(Filter {
                secs: 0,
                micros: 0,
                id: Id::Standard(StandardId::new(0x123).unwrap()),
                dlc: 1,
                data: Vec::from_slice(&[0xFF]).unwrap(),
            })
        );
    }

    #[test]
    fn parse_echo() {
        let (_, result) = command("< echo >").unwrap();
        assert_eq!(result, Command::Echo(Echo));
    }

    #[test]
    fn parse_raw_mode() {
        let (_, result) = command("< rawmode >").unwrap();
        assert_eq!(result, Command::RawMode(RawMode));
    }

    #[test]
    fn parse_broadcast_mode() {
        let (_, result) = command("< bcmode >").unwrap();
        assert_eq!(result, Command::BroadcastMode(BroadcastMode));
    }

    #[test]
    fn parse_control_mode() {
        let (_, result) = command("< controlmode >").unwrap();
        assert_eq!(result, Command::ControlMode(ControlMode));
    }

    #[test]
    fn statistics() {
        let (_, result) = command("< statistics 1000 >").unwrap();
        assert_eq!(
            result,
            Command::Statistics(Statistics {
                interval_millis: 1000
            })
        );
    }
}
