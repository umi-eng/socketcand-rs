use core::num::ParseIntError;
use core::str::FromStr;
use heapless::Vec;
use nom::{
    branch::alt,
    bytes::{complete::tag, streaming::take_while1},
    character::{
        complete::char,
        streaming::{digit1, hex_digit1, space1},
    },
    combinator::{map, map_res},
    sequence::{delimited, terminated, tuple},
    IResult,
};

fn from_hex(input: &str) -> Result<u8, ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

/// Frame job add command.
#[derive(Debug, PartialEq)]
struct Add {
    interval_secs: u32,
    interval_micros: u32,
    id: u32,
    dlc: u8,
    data: Vec<u8, 64>,
}

impl Add {
    /// Parse ASCII command.
    ///
    /// Format:
    /// `< add interval_secs interval_micros id dlc data0 data1 dataN... >`
    fn parse_from_ascii(input: &str) -> IResult<&str, Self> {
        let (input, (_, interval_secs, interval_micros, id, dlc, data)) =
            delimited(
                tuple((char('<'), space1)),
                tuple((
                    tag("add "),
                    terminated(map_res(digit1, u32::from_str), char(' ')),
                    terminated(map_res(digit1, u32::from_str), char(' ')),
                    terminated(
                        map_res(hex_digit1, |id: &str| {
                            u32::from_str_radix(id, 16)
                        }),
                        char(' '),
                    ),
                    terminated(map_res(digit1, u8::from_str), char(' ')),
                    map(
                        take_while1(|c: char| c.is_digit(16) || c == ' '),
                        |bytes: &str| {
                            bytes
                                .split_whitespace()
                                .filter_map(|b| u8::from_str_radix(b, 16).ok())
                                .collect::<Vec<u8, 64>>()
                        },
                    ),
                )),
                char('>'),
            )(input)?;

        Ok((
            input,
            Self {
                interval_secs,
                interval_micros,
                id,
                dlc,
                data,
            },
        ))
    }
}

/// Frame job update command.
#[derive(Debug)]
struct Update {
    id: u32,
    dlc: u8,
    data: Vec<u8, 64>,
}

impl Update {
    /// Parse ASCII command.
    ///
    /// Format:
    /// `< update id dlc data0 data1 dataN... >`
    fn parse_from_ascii(input: &str) -> IResult<&str, Self> {
        let (input, (_, id, dlc, data)) = delimited(
            tuple((char('<'), space1)),
            tuple((
                tag("add "),
                terminated(
                    map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                    char(' '),
                ),
                terminated(map_res(digit1, u8::from_str), char(' ')),
                map(
                    take_while1(|c: char| c.is_digit(16) || c == ' '),
                    |bytes: &str| {
                        bytes
                            .split_whitespace()
                            .filter_map(|b| u8::from_str_radix(b, 16).ok())
                            .collect::<Vec<u8, 64>>()
                    },
                ),
            )),
            char('>'),
        )(input)?;

        Ok((input, Update { id, dlc, data }))
    }
}

/// Frame job delete command.
#[derive(Debug)]
struct Delete {
    id: u32,
}

impl Delete {
    fn parse_from_ascii(input: &str) -> IResult<&str, Self> {
        let (input, (_, id)) = delimited(
            tuple((char('<'), space1)),
            tuple((
                tag("add "),
                terminated(
                    map_res(hex_digit1, |id: &str| u32::from_str_radix(id, 16)),
                    char(' '),
                ),
            )),
            char('>'),
        )(input)?;

        Ok((input, Delete { id }))
    }
}

/// Single frame send command.
#[derive(Debug)]
struct Send {
    id: u32,
    dlc: u8,
    data: Vec<u8, 64>,
}

impl Send {
    fn parse_from_ascii(input: &str) -> IResult<&str, Self> {
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
                    take_while1(|c: char| c.is_digit(16) || c == ' '),
                    |bytes: &str| {
                        bytes
                            .split_whitespace()
                            .filter_map(|b| u8::from_str_radix(b, 16).ok())
                            .collect::<Vec<u8, 64>>()
                    },
                ),
            )),
            char('>'),
        )(input)?;

        Ok((input, Send { id, dlc, data }))
    }
}

#[derive(Debug)]
enum Command {
    Add(Add),
    Update(Update),
    Delete(Delete),
    Send(Send),
}

impl From<Add> for Command {
    fn from(value: Add) -> Self {
        Command::Add(value)
    }
}

impl From<Update> for Command {
    fn from(value: Update) -> Self {
        Command::Update(value)
    }
}

impl From<Delete> for Command {
    fn from(value: Delete) -> Self {
        Command::Delete(value)
    }
}

impl From<Send> for Command {
    fn from(value: Send) -> Self {
        Command::Send(value)
    }
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    alt((
        |i| Add::parse_from_ascii(i).map(|out| (out.0, out.1.into())),
        |i| Update::parse_from_ascii(i).map(|out| (out.0, out.1.into())),
        |i| Delete::parse_from_ascii(i).map(|out| (out.0, out.1.into())),
        |i| Send::parse_from_ascii(i).map(|out| (out.0, out.1.into())),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        assert_eq!(from_hex("5"), Ok(5));
        assert_eq!(from_hex("0"), Ok(0));
        assert_eq!(from_hex("F"), Ok(15));
    }

    #[test]
    fn test_parse_add() {
        // todo: this should really be done with a fuzzer.
        let cases = [
            (
                "< add 1 0 123 8 11 22 33 44 55 66 77 88 >",
                Add {
                    interval_secs: 1,
                    interval_micros: 0,
                    id: 0x123,
                    dlc: 8,
                    data: Vec::from_slice(&[
                        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                    ])
                    .unwrap(),
                },
            ),
            (
                "< add 1 0 123 8 1 2 3 4 5 6 7 8 >",
                Add {
                    interval_secs: 1,
                    interval_micros: 0,
                    id: 0x123,
                    dlc: 8,
                    data: Vec::from_slice(&[
                        0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8,
                    ])
                    .unwrap(),
                },
            ),
            (
                "< add 1 0 123 5 1 2 3 4 5 >< more >< echo >",
                Add {
                    interval_secs: 1,
                    interval_micros: 0,
                    id: 0x123,
                    dlc: 5,
                    data: Vec::from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]).unwrap(),
                },
            ),
        ];

        for case in cases {
            let output = Add::parse_from_ascii(case.0).unwrap();
            let result = output.1;
            let expected = case.1;

            assert_eq!(expected, result);
        }
    }
}
