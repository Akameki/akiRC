mod command_parse;
mod nom_extended;
mod sub_parse;

use std::str::FromStr;

use command_parse::parse_command;
use nom::{
    Finish, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, none_of, one_of},
    combinator::{all_consuming, opt, recognize},
    multi::{many_m_n, many0, many1},
    sequence::{delimited, preceded},
};
use sub_parse::space;

use crate::{IrcError, message::Message};

impl FromStr for Message {
    type Err = IrcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_message(s)
    }
}

pub fn parse_message(i: &str) -> Result<Message, IrcError> {
    let i = i.trim();
    let (_, (pref, cmd, params)) = all_consuming((opt(prefix), command, params))
        .parse(i)
        .finish()
        .map_err(|e| IrcError::IrcParseError(format!("[{}] @ parse_message(\"{}\")", e, i)))?;

    let prefix = pref.map(|p| p.to_owned());
    let command = parse_command(cmd, &params);

    Ok(Message { prefix: prefix.map(|p| p.to_owned()), command })
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
    } else {
        // : is optional
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
