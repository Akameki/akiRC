#![allow(dead_code)]

use nom::{
    branch::{alt, Choice}, bytes::complete::{tag, take}, character::complete::{alphanumeric1, char, none_of, one_of, satisfy, space1}, combinator::{opt, recognize, verify}, multi::{many1, many_m_n, separated_list1}, sequence::terminated, Err, IResult, Parser
};

use super::nom_extended::{str_one_of, take_until_one_of};

// todo: consider not counting \t as space?
pub fn space(i: &str) -> IResult<&str, &str> {
    space1(i)
}

// fn debugp(str: &str) -> IResult<&str, &str> {
//     println!("{str}");
//     Err(Err::Error(nom::error::Error {input: "debug", code: nom::error::ErrorKind::Eof}))
// }

// "targets"
pub fn msgto(i: &str) -> IResult<&str, &str> {
    // channel / ( user [ "%" host ] "@" servername ) / ( user "%" host ) / targetmask / nickname / (nickname "!" user "@" host )
    alt((
        channel,
        recognize((nickname, tag("!"), user, tag("@"), host)),
        recognize((user, opt((tag("%"), host)), tag("@"), servername)),
        recognize((user, tag("%"), host)),
        // todo targetmask,
        nickname,
    )).parse(i)
}
pub fn channel(i: &str) -> IResult<&str, &str> {
    recognize((
        alt((str_one_of("#+&"), recognize((str_one_of("!"), channelid)))),
        chanstring,
    ))
    .parse(i)
    // TODO: [:chanstring] mask
}
pub fn servername(i: &str) -> IResult<&str, &str> {
    hostname(i)
}
pub fn host(i: &str) -> IResult<&str, &str> {
    // hostname / host addr
    alt((hostname, hostaddr)).parse(i)
}
pub fn hostname(i: &str) -> IResult<&str, &str> {
    // shortname *( "." shortname )
    recognize(separated_list1(char('.'), shortname)).parse(i)
}
pub fn shortname(i: &str) -> IResult<&str, &str> {
    // alphanumeric *( [ "-" ] alphanumeric )
    recognize(separated_list1(one_of("-"), alphanumeric1)).parse(i)
}
pub fn hostaddr(i: &str) -> IResult<&str, &str> {
    // ip4addr / ip6addr
    recognize(many1(one_of("0123456789abdefABCDEF:."))).parse(i)
}
// pub fn ip4addr(i: &str) -> IResult<&str, &str> {
//     // 1*3digit "." 1*3digit "." 1*3digit "." 1*3digit
//     recognize(separated_list1(char('.'), recognize(many_m_n(1, 3, one_of("0123456789"))))).parse(i)
// }
// pub fn ip6addr(i &str) -> IResult<&str, &str> {

// }
pub fn nickname(i: &str) -> IResult<&str, &str> {
    recognize((alt((letter, digit)), many_m_n(0,15, alt((letter, digit, special, tag("-")))))).parse(i)
}
pub fn chanstring(i: &str) -> IResult<&str, &str> {
    take_until_one_of("\0\x07\r\n ,:")(i)
}
pub fn channelid(i: &str) -> IResult<&str, &str> {
    verify(take(5usize), |x: &str| {
        x.chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
    })
    .parse(i)
}

// "Other parameters syntaxes"
pub fn user(i: &str) -> IResult<&str, &str> {
    recognize(many1(none_of("\x00\x0A\x0D\x20\x40%"))).parse(i)
}
pub fn key(i: &str) -> IResult<&str, &str> {
    let is_valid = |c: char| {
        matches!(c as u8, 0x01..=0x05 | 0x07..=0x08 | 0x0C | 0x0E..=0x1F | 0x21..=0x7F) && c != ','
    };
    recognize(many_m_n(1, 23, satisfy(is_valid))).parse(i)
}
pub fn letter(i: &str) -> IResult<&str, &str> {
    str_one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz").parse(i)
}
pub fn digit(i: &str) -> IResult<&str, &str> {
    str_one_of("0123456789").parse(i)
}
pub fn special(i: &str) -> IResult<&str, &str> {
    str_one_of("[]\\'_^{|}").parse(i)
}

// "Wildcard Expressions" - the ABNF seems strange..
pub fn mask(i: &str) -> IResult<&str, &str> {
    recognize(many1(alt((
        none_of("\0*?"),
        terminated(none_of("\0\\"), one_of("?*")),
    ))))
    .parse(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_targets() {
        assert_eq!(user("user@"), Ok(("@", "user")));
        assert_eq!(host("host- "), Ok(("- ", "host")));
        assert_eq!(nickname("nickname "), Ok((" ", "nickname")));
        assert_eq!(msgto("#hello world"), Ok((" world", "#hello")));
        assert_eq!(msgto("!AB123name@host "), Ok((" ", "!AB123name@host")));
        assert_eq!(msgto("user%host@servername "), Ok((" ", "user%host@servername")));
        assert_eq!(msgto("user%host "), Ok((" ", "user%host")));
        assert_eq!(msgto("nickname!user@host "), Ok((" ", "nickname!user@host")));
        assert_eq!(msgto("nickname "), Ok((" ", "nickname")));
    }

    #[test]
    fn test_channel_related() {
        assert_eq!(channel("#hello world"), Ok((" world", "#hello")));
        assert_eq!(channel("++hello"), Ok(("", "++hello")));
        assert_eq!(channel("!AB123name\n"), Ok(("\n", "!AB123name")));
        assert!(channel("hello").is_err());
    }
}
