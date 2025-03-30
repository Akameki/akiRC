use nom::{
    IResult, Parser,
    character::complete::{none_of, one_of},
    combinator::recognize,
    multi::many1,
};

pub fn str_one_of(list: &str) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |i: &str| one_of(list).parse(i).map(|(i2, _)| (i2, &i[..1]))
}

pub fn take_until_one_of(list: &str) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |i: &str| recognize(many1(none_of(list))).parse(i)
}

/// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_one_of() {
        assert_eq!(str_one_of("abc")("a123"), Ok(("123", "a")));
        assert!(str_one_of("abc")("123").is_err());
    }

    #[test]
    fn test_take_until_one_of() {
        assert_eq!(take_until_one_of("abc")("123b456"), Ok(("b456", "123")));
        assert_eq!(take_until_one_of("abc")("123"), Ok(("", "123")));
        assert!(take_until_one_of("abc")("").is_err());
        assert!(take_until_one_of("abc")("c").is_err());
    }
}
