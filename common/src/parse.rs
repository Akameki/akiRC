mod command_parse;
mod nom_extended;
mod sub_parse;

use std::str::FromStr;

use nom::{
    Finish, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, none_of, one_of},
    combinator::{all_consuming, opt, recognize},
    multi::{many_m_n, many0, many1},
    sequence::{delimited, preceded},
};

use crate::{
    IrcError,
    message::{Command, Message},
};
use command_parse::parse_list;

impl FromStr for Message {
    type Err = IrcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let command = Command::new(cmd, params); // todo: use Command::from_str to parse args

        let message = parse_message(s)?;

        if matches!(message.command, Command::Invalid) {
            Err(IrcError::IrcParseError(format!(
                "Unrecognized command {}",
                s
            )))
        } else {
            Ok(message)
        }
    }
}

pub fn parse_message(i: &str) -> Result<Message, IrcError> {
    let (_, (pref, cmd, params)) = all_consuming((opt(prefix), command, params))
        .parse(i)
        .finish()
        .map_err(|e| IrcError::IrcParseError(format!("Error {} parsing {}", e, i)))?;

    let prefix = pref.map(|p| p.to_owned());
    let command = parse_command(cmd, &params)
        .map_err(|e| {
            if let IrcError::IrcParseError(e) = e {
                IrcError::IrcParseError(format!(
                    "While parsing message {},\nfailed to parse commmand {} with error {}",
                    i, cmd, e
                ))
            } else {
                unreachable!()
            }
        })
        .unwrap();

    Ok(Message {
        prefix: prefix.map(|p| p.to_owned()),
        command,
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

fn parse_command(cmd: &str, params: &[&str]) -> Result<Command, IrcError> {
    use Command::*;

    // let (cmd, params) = i;
    let len = params.len();

    let mut params_iter = params.iter().cloned();
    /// Consumes the next &str in params_iter, returning it as an owned String.
    /// Meant to be a "default" when formal parsing is not yet implemented.
    macro_rules! req {
        () => {
            params_iter.next().unwrap().to_owned()
        };
    }
    // TODO: validate parameters where needed
    match cmd {
        "NICK" => Ok(Nick(req!())),
        "USER" if len >= 4 => Ok(User(req!(), req!(), req!(), req!())),
        "LIST" => parse_list(params),
        _ => Ok(Command::Invalid),
    }
}
