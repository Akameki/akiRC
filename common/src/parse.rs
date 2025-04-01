mod command_parse;
mod nom_extended;
mod sub_parse;

use std::str::FromStr;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::{alpha1, char, none_of, one_of}, combinator::{all_consuming, opt, recognize}, multi::{many0, many1, many_m_n}, sequence::{delimited, preceded}, Finish, IResult, Parser
};

use crate::{
    IrcError,
    message::{Command, Message},
};

use sub_parse::space;
use command_parse::parse_command;

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
    let i = i.trim();
    let (_, (pref, cmd, params)) = all_consuming((opt(prefix), command, params))
        .parse(i)
        .finish()
        .map_err(|e| IrcError::IrcParseError(format!("[{}] @ parse_message(\"{}\")", e, i)))?;

    let prefix = pref.map(|p| p.to_owned());
    let command = parse_command(cmd, &params)
        .map_err(|e| {
            if let IrcError::IrcParseError(e) = e {
                IrcError::IrcParseError(format!(
                    "While parsing message ({})...\nfailed to parse commmand {} with error {}",
                    i, cmd, e
                ))
            } else {
                unreachable!()
            }
        })?;

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

    let (i, mut params) = many_m_n(0, 14, preceded(space, middle)).parse(i)?;

    let (i, trail) = if params.len() < 14 {
        opt(preceded((space, tag(":")), trailing)).parse(i)?
    } else { // : is optional
        opt(preceded((space, opt(tag(":"))), trailing)).parse(i)?
    };
    if let Some(t) = trail {
        params.push(t);
    }
    Ok((i, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Command;

    macro_rules! stringvec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn test_parse_message() {
        assert_eq!(
            parse_message(":test!user@host NICK test").unwrap(),
            Message::new(
                Some("test!user@host"),
                Command::Nick("test".to_string())
            )
        );
        assert_eq!(
            parse_message("NICK :test").unwrap(),
            Message::new(None, Command::Nick("test".to_string()))
        );
        assert_eq!(
            parse_message(":pref NICK test with extra").unwrap(),
            Message::new(
                Some("pref"),
                Command::Nick("test".to_string())
            )
        );
    }
}