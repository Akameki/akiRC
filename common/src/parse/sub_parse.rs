use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::take,
    combinator::{recognize, verify},
};

use super::nom_extended::{str_one_of, take_until_one_of};

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
