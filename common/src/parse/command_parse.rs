use nom::{Parser, bytes::complete::tag, combinator::all_consuming, multi::separated_list1};

use super::sub_parse::channel;
use crate::{
    message::{Command, Numeric::*},
    parse::sub_parse::{nickname, user},
};

pub fn parse_command(cmd: &str, params: &[&str]) -> Command {
    match cmd.to_uppercase().as_str() {
        "NICK" => parse_NICK(params),
        "USER" => parse_USER(params),
        /* Channel Operations */
        "JOIN" => parse_JOIN(params),
        "PART" => parse_PART(params),
        "LIST" => parse_LIST(params),
        "WHO" => parse_WHO(params),
        "PRIVMSG" => parse_PRIVMSG(params),
        _ => Command::Invalid(
            cmd.to_string(),
            Some(ERR_UNKNOWNCOMMAND),
            format!("{cmd} :Unknown command"),
        ),
    }
}

/* Utility functions/macros */
// /// Returns Invalid if given parameter list is not at least some length.
// /// ### Example
// /// ```ignore
// /// check_need_more_params!("NICK", params, 1);
// /// ```
// macro_rules! check_need_more_params {
//     ($cmd:literal, $params:expr, $len:expr) => {
//         if $params.len() < $len {
//             return Command::Invalid(
//                 $cmd.to_string(),
//                 Some(ERR_NEEDMOREPARAMS),
//                 format!("{} :Not enough parameters", $cmd),
//             );
//         }
//     };
// }

/// Returns output of a parser if completely parsed, or otherwise returns a given Command as error.
/// ### Example
/// ```ignore
/// let nick = parse_or_return_invalid!("NICK", nickname, params[0], ERR_ERRONEUSNICKNAME, format!("{} :Erroneus nickname", params[0]));
/// ```
macro_rules! parse_or_return_invalid {
    ($cmd:literal, $parser:expr, $param:expr, $numeric:expr, $str:expr) => {{
        if let Ok((_, parsed)) = all_consuming($parser).parse($param) {
            parsed
        } else {
            return Command::Invalid($cmd.to_string(), Some($numeric), $str);
        }
    }};
}

/// Returns output of a parser if completely parsed, or otherwise returns a default.
/// ### Example
/// ```ignore
/// let nick = parse_or_default!("NICK", nickname, params[0], "dummynick");
/// ```
macro_rules! parse_or_default {
    ($parser:expr, $param:expr, $default:expr) => {{ if let Ok((_, parsed)) = all_consuming($parser).parse($param) { parsed } else { $default } }};
}

// Individual command parsers below.
// Extra parameters are ignored.

#[allow(non_snake_case)]
fn parse_NICK(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "NICK".to_string(),
            Some(ERR_NONICKNAMEGIVEN),
            ":No nickname given".to_string(),
        );
    }
    let nickname = parse_or_return_invalid!(
        "NICK",
        nickname,
        params[0],
        ERR_ERRONEUSNICKNAME,
        format!("{} :Erroneus nickname", params[0])
    )
    .to_owned();
    Command::NICK { nickname }
}

#[allow(non_snake_case)]
fn parse_USER(params: &[&str]) -> Command {
    if params.len() < 4 {
        return Command::Invalid(
            "USER".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            format!("{} :Not enough parameters", "USER"),
        );
    }
    let username = parse_or_default!(user, params[0], "!INVALID").to_owned();
    let a = params[1].to_owned();
    let b = params[2].to_owned();
    let realname = params[3].to_owned();
    Command::USER { username, _1: a, _2: b, realname }
}

/* Channel Operations */

// irc.libera.chat does not behave as expected when passing multiple channels..
#[allow(non_snake_case)]
fn parse_JOIN(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "JOIN".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            "JOIN :Not enough parameters".to_string(),
        );
    }
    if params[0] == "0" {
        return Command::JOIN { channels: vec![], keys: vec![], alt: true };
    }

    let channels = params[0].split(",").map(String::from).collect();
    // separated_list1(tag(","), channel),

    let keys = if params.len() >= 2 {
        params[1].split(",").map(String::from).collect()
    } else {
        Vec::new()
    };
    // separated_list1(tag(","), key)

    Command::JOIN { channels, keys, alt: false }
}

#[allow(non_snake_case)]
fn parse_PART(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "PART".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            "PART :Not enough parameters".to_string(),
        );
    }
    let channels = params[0].split(",").map(String::from).collect();
    // separated_list1(tag(","), channel)

    let reason = if params.len() >= 2 { params[1].to_owned() } else { "".to_string() };

    Command::PART { channels, reason }
}

#[allow(non_snake_case)]
fn parse_LIST(params: &[&str]) -> Command {
    let channels = if !params.is_empty() {
        params[0].split(",").map(String::from).collect()
        // separated_list1(tag(","), channel)
    } else {
        Vec::new()
    };

    let elistconds = None;

    Command::LIST { channels, elistconds }
}

#[allow(non_snake_case)]
fn parse_WHO(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "WHO".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            "WHO :Not enough parameters".to_string(),
        );
    }
    // RFC has some weird syntax for masks. irc.libera.chat accepts anything.
    // let (_, mask) = all_consuming(mask).parse(params[0])?;
    let mask = params[0].to_owned();
    Command::WHO { mask }
}

#[allow(non_snake_case)]
fn parse_PRIVMSG(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "PRIVMSG".to_string(),
            Some(ERR_NORECIPIENT),
            ":No recipient given (PRIVMSG)".to_string(),
        );
    } else if params.len() < 2 {
        return Command::Invalid(
            "PRIVMSG".to_string(),
            Some(ERR_NOTEXTTOSEND),
            ":No text to send".to_string(),
        );
    }
    let targets = params[0].split(",").map(String::from).collect();
    // let (_, targets) = all_consuming(separated_list1(tag(","), msgto)).parse(params[0])?;
    // let targets = targets.into_iter().map(|x| x.to_string()).collect();
    let text = params[1].to_owned();
    Command::PRIVMSG { targets, text }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! stringvec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn test_nick() {
        assert_eq!(
            parse_NICK(&[]),
            Command::Invalid(
                "NICK".to_string(),
                Some(ERR_NONICKNAMEGIVEN),
                ":No nickname given".to_string()
            )
        );
        assert_eq!(
            parse_NICK(&["mynick", "extra"]),
            Command::NICK { nickname: "mynick".to_string() }
        );
        assert_eq!(
            parse_NICK(&["0invalid"]),
            Command::Invalid(
                "NICK".to_string(),
                Some(ERR_ERRONEUSNICKNAME),
                "0invalid :Erroneus nickname".to_string()
            )
        );
    }

    #[test]
    fn test_join() {
        assert_eq!(
            parse_JOIN(&[]),
            Command::Invalid(
                "JOIN".to_string(),
                Some(ERR_NEEDMOREPARAMS),
                "JOIN :Not enough parameters".to_string()
            )
        );
        assert_eq!(
            parse_JOIN(&["#chan1"]),
            Command::JOIN { channels: stringvec!["#chan1"], keys: vec![], alt: false }
        );
        assert_eq!(
            parse_JOIN(&["#chan1,#chan2,#chan3", "key1,key2"]),
            Command::JOIN {
                channels: stringvec!["#chan1", "#chan2", "#chan3"],
                keys: stringvec!["key1", "key2"],
                alt: false
            }
        );
        assert_eq!(parse_JOIN(&["0"]), Command::JOIN { channels: vec![], keys: vec![], alt: true });
    }
    #[test]
    fn test_list() {
        assert_eq!(parse_LIST(&[]), Command::LIST { channels: stringvec![], elistconds: None });
        assert_eq!(
            parse_LIST(&["#chan1"]),
            Command::LIST { channels: vec!["#chan1".to_string()], elistconds: None }
        );
        assert_eq!(
            parse_LIST(&["#chan1,#chan2,#chan3"]),
            Command::LIST { channels: stringvec!["#chan1", "#chan2", "#chan3"], elistconds: None }
        );
    }
}
