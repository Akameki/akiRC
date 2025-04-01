use nom::{
    branch::alt, bytes::complete::take, character::complete::{satisfy, space1}, combinator::{recognize, verify}, multi::many_m_n, IResult, Parser
};

use super::nom_extended::{str_one_of, take_until_one_of};

// todo: consider not counting \t as space?
pub fn space(i: &str) -> IResult<&str, &str> {
    space1(i)
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
pub fn channel(i: &str) -> IResult<&str, &str> {
    recognize((
        alt((str_one_of("#+&"), recognize((str_one_of("!"), channelid)))),
        chanstring,
    ))
    .parse(i)
    // TODO: [:chanstring] mask
}

// "Other parameters"
pub fn key(i: &str) -> IResult<&str, &str> {
    let is_valid =
        |c: char| matches!(c as u8, 0x01..=0x05 | 0x07..=0x08 | 0x0C | 0x0E..=0x1F | 0x21..=0x7F) && c!= ',';
    recognize(many_m_n(1, 23, satisfy(is_valid))).parse(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_related() {
        assert_eq!(channel("#hello world"), Ok((" world", "#hello")));
        assert_eq!(channel("++hello"), Ok(("", "++hello")));
        assert_eq!(channel("!AB123name\n"), Ok(("\n", "!AB123name")));
        assert!(channel("hello").is_err());
    }
}
