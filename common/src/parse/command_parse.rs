use nom::{Parser, bytes::complete::tag, combinator::all_consuming, multi::separated_list1};

use super::sub_parse::{channel, key, mask, msgto};
use crate::{IrcError, message::Command};

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
        "NICK" if len >= 1 => Ok(NICK { nickname: req!() }),
        "USER" if len >= 4 => {
            Ok(USER { username: req!(), _1: req!(), _2: req!(), realname: req!() })
        }
        "JOIN" => parse_JOIN(params),
        "LIST" => parse_LIST(params),
        "WHO" => parse_WHO(params),
        "PRIVMSG" => parse_PRIVMSG(params),
        _ => Ok(Command::Invalid),
    }
}

#[allow(non_snake_case)]
fn parse_JOIN(ps: &[&str]) -> Result<Command, IrcError> {
    if ps.is_empty() {
        return Err(IrcError::IrcParseError("NEEDMOREPARAMS".to_string()));
    } else if ps[0] == "0" {
        return Ok(Command::JOIN { channels: vec![], keys: vec![], alt: true });
    }
    let (_, chs) = all_consuming(separated_list1(tag(","), channel)).parse(ps[0])?;
    let channels = chs.into_iter().map(|x| x.to_string()).collect();
    let keys = if ps.len() >= 2 {
        let (_, ks) = all_consuming(separated_list1(tag(","), key)).parse(ps[1])?;
        ks.into_iter().map(|x| x.to_string()).collect()
    } else {
        Vec::new()
    };
    Ok(Command::JOIN { channels, keys, alt: false })
}

#[allow(non_snake_case)]
pub fn parse_LIST(ps: &[&str]) -> Result<Command, IrcError> {
    let mut channels = Vec::new();
    let elistconds = None;
    if !ps.is_empty() {
        let (_, chs) = all_consuming(separated_list1(tag(","), channel)).parse(ps[0])?;
        channels = chs.into_iter().map(|x| x.to_string()).collect();
    }

    Ok(Command::LIST { channels, elistconds })
}

#[allow(non_snake_case)]
pub fn parse_WHO(ps: &[&str]) -> Result<Command, IrcError> {
    if ps.is_empty() {
        return Err(IrcError::IrcParseError("NEEDMOREPARAMS".to_string()));
    }
    let (_, mask) = all_consuming(mask).parse(ps[0])?;

    Ok(Command::WHO { mask: mask.to_owned() })
}

#[allow(non_snake_case)]
pub fn parse_PRIVMSG(ps: &[&str]) -> Result<Command, IrcError> {
    if ps.len() < 2 {
        return Err(IrcError::IrcParseError("NEEDMOREPARAMS".to_string()));
    }
    let (_, targets) = all_consuming(separated_list1(tag(","), msgto)).parse(ps[0])?;
    let targets = targets.into_iter().map(|x| x.to_string()).collect();
    let text = ps[1].to_owned();
    Ok(Command::PRIVMSG { targets, text })
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
            parse_JOIN(&["#chan1,#chan2,#chan3"]).unwrap(),
            Command::JOIN {
                channels: stringvec!["#chan1", "#chan2", "#chan3"],
                keys: vec![],
                alt: false
            }
        );
        assert_eq!(
            parse_JOIN(&["#chan1,#chan2,#chan3", "key1,key2"]).unwrap(),
            Command::JOIN {
                channels: stringvec!["#chan1", "#chan2", "#chan3"],
                keys: stringvec!["key1", "key2"],
                alt: false
            }
        );
        assert_eq!(
            parse_JOIN(&["0"]).unwrap(),
            Command::JOIN { channels: vec![], keys: vec![], alt: true }
        );
        assert!(parse_JOIN(&[]).is_err());
        assert!(parse_JOIN(&["uh"]).is_err());
    }
    #[test]
    fn test_list() {
        assert_eq!(
            parse_LIST(&["#chan1,#chan2,#chan3"]).unwrap(),
            Command::LIST { channels: stringvec!["#chan1", "#chan2", "#chan3"], elistconds: None }
        );
        assert_eq!(
            parse_LIST(&["#chan1"]).unwrap(),
            Command::LIST { channels: vec!["#chan1".to_string()], elistconds: None }
        );
        assert_eq!(parse_LIST(&[]).unwrap(), Command::LIST { channels: vec![], elistconds: None });
        assert!(parse_LIST(&["uh"]).is_err());
    }
}
