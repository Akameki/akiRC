use nom::{Parser, combinator::all_consuming};

use crate::{
    message::{Command, Numeric::*},
    parse::sub_parse::{nickname, user},
};

pub fn parse_command(cmd: &str, params: &[&str]) -> Command {
    match cmd.to_uppercase().as_str() {
        /* Connection Messages */
        // CAP
        // AUTHENTICATE
        // PASS
        "NICK" => parse_NICK(params),
        "USER" => parse_USER(params),
        "PING" => parse_PING(params),
        "PONG" => parse_PONG(params),
        // OPER
        "QUIT" => parse_QUIT(params),
        // ERROR

        /* Channel Operations */
        "JOIN" => parse_JOIN(params),
        "PART" => parse_PART(params),
        "TOPIC" => parse_TOPIC(params),
        // NAMES
        "LIST" => parse_LIST(params),
        // INVITE
        // KICK

        /* Server Queries and Commands */
        "MOTD" => parse_MOTD(params),
        // VERSION
        // ADMIN
        // CONNECT
        // LUSERS
        // TIME
        // STATS
        // HELP
        // INFO
        "MODE" => parse_MODE(params),

        /* Sending Messages */
        "PRIVMSG" => parse_PRIVMSG(params),
        // NOTICE

        /* User Based Queries */
        "WHO" => parse_WHO(params),
        // WHOIS
        // WHOWAS

        /* Operator Messages */
        // KILL
        // REHASH
        // RESTART
        // SQUIT

        /* Optional Messages */
        // AWAY
        // LINKS
        // USERHOST
        // WALLOPS

        /* Other */
        _ => Command::Invalid(
            cmd.to_string(),
            Some(ERR_UNKNOWNCOMMAND),
            format!("{cmd} :Unknown command"),
        ),
    }
}

/* Utility functions/macros */

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

/* Connection Messages */
// CAP
// AUTHENTICATE
// PASS
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
    // "" is a sentinal value.
    let username = parse_or_default!(user, params[0], "").to_owned();
    let realname = params[3].to_owned();
    Command::USER { username, _1: (), _2: (), realname }
}
#[allow(non_snake_case)]
fn parse_PING(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "PING".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            "PING :Not enough parameters".to_string(),
        );
    }
    let token = params[0].to_owned();
    Command::PING { token }
}
#[allow(non_snake_case)]
fn parse_PONG(_params: &[&str]) -> Command {
    // no need to parse PONG
    let server = String::new();
    let token = String::new();
    Command::PONG { server, token }
}
// OPER
#[allow(non_snake_case)]
fn parse_QUIT(params: &[&str]) -> Command {
    let reason = if params.is_empty() { "".to_string() } else { params[0].to_string() };
    Command::QUIT { reason }
}
// ERROR

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
fn parse_TOPIC(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "TOPIC".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            "TOPIC :Not enough parameters".to_string(),
        );
    }
    let channel = params[0].to_owned();
    let topic = params.get(1).cloned().map(String::from);

    Command::TOPIC { channel, topic }
}
// NAMES
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
// INVITE
// KICK

/* Server Queries and Commands */
#[allow(non_snake_case)]
fn parse_MOTD(params: &[&str]) -> Command {
    let target = params.first().cloned().unwrap_or_default().to_string();
    Command::MOTD { target }
}
// VERSION
// ADMIN
// CONNECT
// LUSERS
// TIME
// STATS
// HELP
// INFO
#[allow(non_snake_case)]
fn parse_MODE(params: &[&str]) -> Command {
    if params.is_empty() {
        return Command::Invalid(
            "MODE".to_string(),
            Some(ERR_NEEDMOREPARAMS),
            "MODE :Not enough parameters".to_string(),
        );
    }
    let target = params[0].to_owned();
    let mut modestring = String::new();
    let mut modeargs: Vec<String> = Vec::new();
    if params.len() >= 2 {
        let mut mode = '+';
        let mut current_mode = ' ';
        let mut num_modes = 0;
        for c in params[1].chars() {
            if c == '+' || c == '-' {
                mode = c
            } else {
                if current_mode != mode {
                    modestring.push(mode);
                    current_mode = mode;
                }
                modestring.push(c);
                num_modes += 1;
            }
        }
        // push remaining params into modeargs, up to # modes
        modeargs.extend(params.iter().skip(2).take(num_modes).map(|s| s.to_string()));
    }

    Command::MODE { target, modestring, modeargs }
}

/* Sending Messages */
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
// NOTICE

/* User Based Queries */
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
// WHOIS
// WHOWAS

/* Operator Messages */
// KILL
// REHASH
// RESTART
// SQUIT

/* Optional Messages */
// AWAY
// LINKS
// USERHOST
// WALLOPS

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! stringvec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn test_extra_params() {
        assert_eq!(parse_NICK(&["nick", "extra"]), Command::NICK { nickname: "nick".to_string() });
    }

    /* Connection Messages */
    // CAP
    // AUTHENTICATE
    // PASS
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
            parse_NICK(&["0invalid"]),
            Command::Invalid(
                "NICK".to_string(),
                Some(ERR_ERRONEUSNICKNAME),
                "0invalid :Erroneus nickname".to_string()
            )
        );
        assert_eq!(parse_NICK(&["nick"]), Command::NICK { nickname: "nick".to_string() })
    }
    #[test]
    fn test_user() {
        assert_eq!(
            parse_USER(&["user", "a", "b", "realname"]),
            Command::USER {
                username: "user".to_string(),
                _1: (),
                _2: (),
                realname: "realname".to_string()
            }
        );
    }
    // PING
    // PONG
    // OPER
    // QUIT
    // ERROR

    /* Channel Operations */

    #[test]
    fn test_join() {
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
    fn test_part() {
        assert_eq!(
            parse_PART(&["#chan1"]),
            Command::PART { channels: stringvec!["#chan1"], reason: "".to_string() }
        );
        assert_eq!(
            parse_PART(&["#chan1,#chan2,#chan3", "reason"]),
            Command::PART {
                channels: stringvec!["#chan1", "#chan2", "#chan3"],
                reason: "reason".to_string()
            }
        );
    }
    #[test]
    fn test_topic() {
        assert_eq!(
            parse_TOPIC(&["#chan1"]),
            Command::TOPIC { channel: "#chan1".to_string(), topic: None }
        );
        assert_eq!(
            parse_TOPIC(&["#chan1", ""]),
            Command::TOPIC { channel: "#chan1".to_string(), topic: Some("".to_string()) }
        );
        assert_eq!(
            parse_TOPIC(&["#chan1", "topic"]),
            Command::TOPIC { channel: "#chan1".to_string(), topic: Some("topic".to_string()) }
        );
    }
    // NAMES
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
    // INVITE
    // KICK

    /* Server Queries and Commands */
    // MOTD
    // VERSION
    // ADMIN
    // CONNECT
    // LUSERS
    // TIME
    // STATS
    // HELP
    // INFO
    #[test]
    fn test_mode() {
        assert_eq!(
            parse_MODE(&["t", "ab", "p1", "p2", "p3"]),
            Command::MODE {
                target: "t".to_string(),
                modestring: "+ab".to_string(),
                modeargs: stringvec!["p1", "p2"],
            }
        );
        assert_eq!(
            parse_MODE(&["t", "-m-+-m-+p--mm+p", "p1", "p2"]),
            Command::MODE {
                target: "t".to_string(),
                modestring: "-mm+p-mm+p".to_string(),
                modeargs: stringvec!["p1", "p2"],
            }
        );
    }

    /* Sending Messages */
    #[test]
    fn test_privmsg() {
        assert_eq!(
            parse_PRIVMSG(&["#chan1", "text"]),
            Command::PRIVMSG { targets: stringvec!["#chan1"], text: "text".to_string() }
        );
        assert_eq!(
            parse_PRIVMSG(&["#chan1,#chan2,user1", "text"]),
            Command::PRIVMSG {
                targets: stringvec!["#chan1", "#chan2", "user1"],
                text: "text".to_string()
            }
        );
    }
    // NOTICE

    /* User Based Queries */
    #[test]
    fn test_who() {
        assert_eq!(parse_WHO(&["#chan1"]), Command::WHO { mask: "#chan1".to_string() });
    }
    // WHOIS
    // WHOWAS

    /* Operator Messages */
    // KILL
    // REHASH
    // RESTART
    // SQUIT

    /* Optional Messages */
    // AWAY
    // LINKS
    // USERHOST
    // WALLOPS
}
