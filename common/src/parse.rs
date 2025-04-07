mod command_parse;
mod nom_extended;
mod sub_parse;

use std::str::FromStr;

use command_parse::parse_command;
use nom::{
    Finish, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, none_of},
    combinator::{all_consuming, map, opt, recognize},
    multi::{many_m_n, many0, many1},
    sequence::{delimited, preceded},
};

use crate::{IrcError, message::Message};

impl FromStr for Message {
    type Err = IrcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_message(s)
    }
}

/// Input should not contain crlf or it will fail.
pub fn parse_message(i: &str) -> Result<Message, IrcError> {
    let i = i.trim();
    let (_, (pref, cmd, params)) = all_consuming((opt(prefix), command, params))
        .parse(i)
        .finish()
        .map_err(|e| IrcError::IrcParseError(format!("[{}] @ parse_message(\"{}\")", e, i)))?;

    let prefix = pref.map(|p| p.to_owned());
    let command = parse_command(cmd, &params);

    Ok(Message { prefix, command })
}

fn prefix(i: &str) -> IResult<&str, &str> {
    let servername = recognize(many1(none_of(" ")));
    delimited(char(':'), servername, space).parse(i)
}
fn command(i: &str) -> IResult<&str, &str> {
    alpha1(i)
}
fn params(i: &str) -> IResult<&str, Vec<&str>> {
    let middle = recognize((nospcrlfcl, many0(alt((tag(":"), nospcrlfcl)))));
    let trailing = recognize(many1(alt((tag(":"), tag(" "), nospcrlfcl))));

    let (i, mut params) = many_m_n(0, 14, preceded(space, middle)).parse(i)?;

    let (i, trail) = if params.len() < 14 {
        opt(preceded((space, tag(":")), recognize(opt(trailing)))).parse(i)?
    } else {
        opt(preceded((space, opt(tag(":"))), trailing)).parse(i)?
    };
    if let Some(t) = trail {
        params.push(t);
    }
    Ok((i, params))
}

fn nospcrlfcl(i: &str) -> IResult<&str, &str> {
    recognize(none_of("\0\r\n :")).parse(i)
}
fn space(i: &str) -> IResult<&str, &str> {
    map(many1(tag(" ")), |spaces| spaces[0]).parse(i)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Command;

    // macro_rules! stringvec {
    //     ($($x:expr),*) => (vec![$($x.to_string()),*]);
    // }

    #[test]
    fn test_parse_message() {
        assert_eq!(
            parse_message(":test!user@host NICK test").unwrap(),
            Message::new(Some("test!user@host"), Command::NICK { nickname: "test".to_string() })
        );
        assert_eq!(
            parse_message("NICK :test").unwrap(),
            Message::new(None, Command::NICK { nickname: "test".to_string() })
        );
        assert_eq!(
            parse_message(":pref NICK test with extra").unwrap(),
            Message::new(Some("pref"), Command::NICK { nickname: "test".to_string() })
        );
    }
}
