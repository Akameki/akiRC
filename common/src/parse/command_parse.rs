use nom::{Parser, bytes::complete::tag, combinator::all_consuming, multi::separated_list1};

use crate::{IrcError, message::Command};

use super::sub_parse::{channel, key};

pub fn parse_command(cmd: &str, params: &[&str]) -> Result<Command, IrcError> {
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
        "JOIN" => parse_join(params),
        "LIST" => parse_list(params),
        _ => Ok(Command::Invalid),
    }
}

fn parse_join(ps: &[&str]) -> Result<Command, IrcError> {
    if ps.is_empty() {
        return Err(IrcError::IrcParseError("NEEDMOREPARAMS".to_string()));
    } else if ps[0] == "0" {
        return Ok(Command::Join(vec![], vec![], true));
    }
    let (_, chs) = all_consuming(separated_list1(tag(","), channel)).parse(ps[0])?;
    let channels = chs.into_iter().map(|x| x.to_string()).collect();
    let keys = if ps.len() >= 2 {
        let (_, ks) = all_consuming(separated_list1(tag(","), key)).parse(ps[1])?;
        ks.into_iter().map(|x| x.to_string()).collect()
    } else {
        Vec::new()
    };
    Ok(Command::Join(channels, keys, false))
}

pub fn parse_list(ps: &[&str]) -> Result<Command, IrcError> {
    let mut channels = None;
    let target = None;
    if !ps.is_empty() {
        let (_, chs) = all_consuming(separated_list1(tag(","), channel)).parse(ps[0])?;
        channels = Some(chs.into_iter().map(|x| x.to_string()).collect());
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
    fn test_join() {
        assert_eq!(
            parse_join(&["#chan1,#chan2,#chan3"]).unwrap(),
            Command::Join(stringvec!["#chan1", "#chan2", "#chan3"], vec![], false)
        );
        assert_eq!(
            parse_join(&["#chan1,#chan2,#chan3", "key1,key2"]).unwrap(),
            Command::Join(
                stringvec!["#chan1", "#chan2", "#chan3"],
                stringvec!["key1", "key2"],
                false
            )
        );
        assert_eq!(
            parse_join(&["0"]).unwrap(),
            Command::Join(vec![], vec![], true)
        );
        assert!(parse_join(&[]).is_err());
        assert!(parse_join(&["uh"]).is_err());
    }
    #[test]
    fn test_list() {
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
