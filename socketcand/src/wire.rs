//! Wire protocol parsing.
use core::str::FromStr;
use heapless::Vec;
use nom::{
    branch::alt,
    bytes::{complete::tag, streaming::take_while},
    character::{
        complete::char,
        streaming::{digit1, hex_digit1, space1},
    },
    combinator::{map, map_res},
    sequence::{delimited, terminated, tuple},
    IResult,
};

/// Open command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Open {
    pub index: u8,
    pub virt: bool,
}

fn open<'a>(input: &'a str) -> IResult<&'a str, Open> {
    let (input, (_, interface_type, index)) = delimited(
        tuple((char('<'), space1)),
        tuple((
            tag("open "),
            alt((tag("can"), tag("vcan"))),
            map_res(digit1, u8::from_str),
        )),
        tuple((space1, char('>'))),
    )(input)?;

    let virt = interface_type == "vcan";

    Ok((input, Open { index, virt }))
}

/// Frame job add command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Add {
    pub interval_secs: u32,
    pub interval_micros: u32,
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 8>,
}

fn add<'a>(input: &'a str) -> IResult<&'a str, Add> {
    let (input, (_, interval_secs, interval_micros, id, dlc, data)) =
        delimited(
            tuple((char('<'), space1)),
            tuple((
                tag("add "),
                terminated(map_res(digit1, u32::from_str), char(' ')),
                terminated(map_res(digit1, u32::from_str), char(' ')),
                terminated(
                    map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                    char(' '),
                ),
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

    Ok((
        input,
        Add {
            interval_secs,
            interval_micros,
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
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 8>,
}

fn update<'a>(input: &'a str) -> IResult<&'a str, Update> {
    let (input, (_, id, dlc, data)) = delimited(
        tuple((char('<'), space1)),
        tuple((
            tag("update "),
            terminated(
                map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                char(' '),
            ),
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
    pub id: u32,
}

fn delete<'a>(input: &'a str) -> IResult<&'a str, Delete> {
    let (input, (_, id)) = delimited(
        tuple((char('<'), space1)),
        tuple((
            tag("delete "),
            terminated(
                map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                char(' '),
            ),
        )),
        char('>'),
    )(input)?;

    Ok((input, Delete { id }))
}

/// Single frame send command.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Send {
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 8>,
}

fn send<'a>(input: &'a str) -> IResult<&'a str, Send> {
    let (input, (_, id, dlc, data)) = delimited(
        tuple((char('<'), space1)),
        tuple((
            tag("send "),
            terminated(
                map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                char(' '),
            ),
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
    pub secs: u32,
    pub micros: u32,
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 8>,
}

fn filter<'a>(input: &'a str) -> IResult<&'a str, Filter> {
    let (input, (_, secs, micros, id, dlc, data)) = delimited(
        tuple((char('<'), space1)),
        tuple((
            tag("filter "),
            terminated(map_res(digit1, u32::from_str), char(' ')),
            terminated(map_res(digit1, u32::from_str), char(' ')),
            terminated(
                map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                char(' '),
            ),
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

    Ok((
        input,
        Filter {
            secs,
            micros,
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
    pub interval_millis: u32,
}

fn statistics<'a>(input: &'a str) -> IResult<&'a str, Statistics> {
    let (input, (_, interval_millis)) = delimited(
        tuple((char('<'), space1)),
        tuple((
            tag("statistics "),
            terminated(map_res(digit1, |v: &str| u32::from_str(v)), char(' ')),
        )),
        char('>'),
    )(input)?;

    Ok((input, Statistics { interval_millis }))
}

/// Command instance.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Command {
    Open(Open),
    Add(Add),
    Update(Update),
    Delete(Delete),
    Send(Send),
    Filter(Filter),
    Echo(Echo),
    RawMode(RawMode),
    BroadcastMode(BroadcastMode),
    ControlMode(ControlMode),
    Statistics(Statistics),
}

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
                interval_secs: 1,
                interval_micros: 0,
                id: 0x123,
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
                id: 0x123,
                dlc: 3,
                data: Vec::from_slice(&[0x11, 0x22, 0x33,]).unwrap(),
            })
        );
    }

    #[test]
    fn parse_delete() {
        let (_, result) = command("< delete 123 >").unwrap();
        assert_eq!(result, Command::Delete(Delete { id: 0x123 }));
    }

    #[test]
    fn parse_send_no_data() {
        let (_, result) = command("< send 123 0 >").unwrap();
        assert_eq!(
            result,
            Command::Send(Send {
                id: 0x123,
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
                id: 0x1AAAAAAA,
                dlc: 2,
                data: Vec::from_slice(&[0x1, 0xf1]).unwrap(),
            })
        );
    }

    #[test]
    fn parse_filter() {
        let (_, result) = command("< filter 0 0 123 1 FF >").unwrap();
        assert_eq!(
            result,
            Command::Filter(Filter {
                secs: 0,
                micros: 0,
                id: 0x123,
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
