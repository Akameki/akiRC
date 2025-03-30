use std::str::FromStr;

use nom::{
    Finish, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, none_of, one_of},
    combinator::{all_consuming, map, opt, recognize},
    multi::{many_m_n, many0, many1},
    sequence::{delimited, preceded},
};

use crate::{message::{Command, Message}, IrcError};

impl FromStr for Command {
    type Err = IrcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!("Command::from_str not implemented: {s}")
    }
}

impl FromStr for Message {
    type Err = IrcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let command = Command::new(cmd, params); // todo: use Command::from_str to parse args

        let message = parse_message(s)?;

        if matches!(message.command, Command::Invalid) {
            Err(IrcError::IrcParseError(
                s.to_owned(),
                String::from("unrecognized command"),
            ))
        } else {
            Ok(message)
        }
    }
}

pub fn parse_message(i: &str) -> Result<Message, IrcError> {
    let (_, (prefix, command, params)) = all_consuming((opt(prefix), command, params))
        .parse(i)
        .finish()
        .map_err(|e| IrcError::IrcParseError(i.to_owned(), e.to_string()))?;

    Ok(Message {
        prefix: prefix.map(|p| p.to_owned()),
        command: Command::new(command, &params),
    })
}

fn prefix(i: &str) -> IResult<&str, &str> {
    let servername = recognize(many1(none_of(" ")));
    delimited(char(':'), servername, char(' ')).parse(i)
}

fn command(i: &str) -> IResult<&str, &str> {
    alpha1(i)
}

fn params(i: &str) -> IResult<&str, Vec<&str>> {
    let spcrflcl = "\0\r\n :";
    let nospcrlfcl = none_of(spcrflcl); // FIXME: move
    let middle = recognize((nospcrlfcl, many0(alt((char(':'), none_of(spcrflcl))))));
    let trailing = recognize(many1(alt((one_of(" :"), none_of(spcrflcl)))));

    let (i, mut params) = many_m_n(0, 14, preceded(char(' '), middle)).parse(i)?;

    let tag = if params.len() == 14 {
        tag(" :")
    } else {
        tag(" ")
    };
    let (i, trail) = opt(preceded(tag, trailing)).parse(i)?;
    if let Some(t) = trail {
        params.push(t);
    }
    Ok((i, params))
}