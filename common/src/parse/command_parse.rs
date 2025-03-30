use nom::{Parser, combinator::all_consuming, multi::many0, sequence::preceded};

use crate::{IrcError, message::Command};

use super::{nom_extended::str_one_of, sub_parse::channel};

pub fn parse_list(i: &[&str]) -> Result<Command, IrcError> {
    let mut channels = None;
    let target = None;
    if !i.is_empty() {
        let (_, (ch_one, ch_rest)) =
            all_consuming((channel, many0(preceded(str_one_of(","), channel)))).parse(i[0])?;
        let mut chs = vec![ch_one.to_owned()];
        for ch in ch_rest {
            chs.push(ch.to_owned())
        }
        channels = Some(chs);
        // TODO: target
    }

    Ok(Command::List(channels, target))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! stringvec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(
            parse_list(&["#chan1,#chan2,#chan3"]).unwrap(),
            Command::List(Some(stringvec!["#chan1", "#chan2", "#chan3"]), None)
        );
        assert_eq!(
            parse_list(&["#chan1"]).unwrap(),
            Command::List(Some(vec!["#chan1".to_string()]), None)
        );
        assert_eq!(parse_list(&[]).unwrap(), Command::List(None, None));
        assert!(parse_list(&["uh"]).is_err());
    }
}
