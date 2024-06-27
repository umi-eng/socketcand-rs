use core::str::FromStr;
use heapless::Vec;
use nom::{
    branch::alt,
    bytes::{
        complete::tag,
        streaming::{take_while, take_while1},
    },
    character::{
        complete::char,
        streaming::{digit1, hex_digit1, space1},
    },
    combinator::{map, map_res},
    sequence::{delimited, terminated, tuple},
    IResult,
};

/// Open command.
#[derive(Debug, PartialEq)]
pub struct Open<'a> {
    pub interface: &'a str,
}

fn open<'a>(input: &'a str) -> IResult<&'a str, Open> {
    let (input, interface) = delimited(
        tuple((char('<'), space1, tag("open"), space1)),
        take_while1(|c: char| !c.is_whitespace() && c != '>'),
        tuple((space1, char('>'))),
    )(input)?;

    Ok((input, Open { interface }))
}

/// Frame job add command.
#[derive(Debug, PartialEq)]
pub struct Add {
    pub interval_secs: u32,
    pub interval_micros: u32,
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 64>,
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
                            .collect::<Vec<u8, 64>>()
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
#[derive(Debug, PartialEq)]
pub struct Update {
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 64>,
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
                        .collect::<Vec<u8, 64>>()
                },
            ),
        )),
        char('>'),
    )(input)?;

    Ok((input, Update { id, dlc, data }))
}

/// Frame job delete command.
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct Send {
    pub id: u32,
    pub dlc: u8,
    pub data: Vec<u8, 64>,
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
                            .collect::<Vec<u8, 64>>()
                    }
                },
            ),
        )),
        char('>'),
    )(input)?;

    Ok((input, Send { id, dlc, data }))
}

#[derive(Debug, PartialEq)]
pub struct Filter {
    secs: u32,
    micros: u32,
    id: u32,
    dlc: u8,
    data: Vec<u8, 64>,
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
                        .collect::<Vec<u8, 64>>()
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

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Open(Open<'a>),
    Add(Add),
    Update(Update),
    Delete(Delete),
    Send(Send),
    Filter(Filter),
}

pub fn command<'a>(input: &'a str) -> IResult<&'a str, Command> {
    alt((
        map(open, Command::Open),
        map(add, Command::Add),
        map(update, Command::Update),
        map(delete, Command::Delete),
        map(send, Command::Send),
        map(filter, Command::Filter),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_open() {
        let (_, result) = command("< open can0 >").unwrap();
        assert_eq!(result, Command::Open(Open { interface: "can0" }));
    }

    #[test]
    fn test_parse_add() {
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
    fn test_parse_update() {
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
    fn test_parse_delete() {
        let (_, result) = command("< delete 123 >").unwrap();
        assert_eq!(result, Command::Delete(Delete { id: 0x123 }));
    }

    #[test]
    fn test_parse_send_no_data() {
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
    fn test_parse_send_with_data() {
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
    fn test_parse_filter() {
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
}
