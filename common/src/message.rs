use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Message {
    // tags
    pub prefix: Option<String>,
    pub command: Command,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    /* Connection Messages */
    // CAP
    // AUTHENTICATE
    // PASS
    /// `<nickname>``
    Nick(String),
    /// `<user> <mode> <unused> <realname>`
    User(String, String, String, String),
    // PING
    // PONG
    // OPER
    // QUIT
    // ERROR
    /* Channel Operations */    
    /// `<channels> [keys]`, `0 flag`
    Join(Vec<String>, Vec<String>, bool),
    // PART
    // TOPIC
    // NAMES
    /// `[<channels> [target]]`
    List(Option<Vec<String>>, Option<String>),
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
    // MODE
    /* Sending Messages */
    // PRIVMSG
    // NOTICE
    /* User Based Queries */
    WHO{mask: String},
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

    /* Non client messages */
    Numeric(Numeric, Vec<String>),
    Invalid,
}

impl Message {
    pub fn new(prefix: Option<&str>, command: Command) -> Self {
        Message {
            prefix: prefix.map(|s| s.to_string()),
            command,
        }
    }
}

// impl Command {
//     pub fn new(command: &str, params: &[&str]) -> Self {
//         use Command::*;

//         let len = params.len();
//         let mut params_iter = params.iter().cloned();

//         macro_rules! req {
//             () => {
//                 params_iter.next().unwrap().to_owned()
//             };
//         }

//         match command {
//             "NICK" => Nick(String::from(params[0])),
//             "USER" if len >= 4 => User(req!(), req!(), req!(), req!()),
//             _ => Invalid,
//         }
//     }
// }

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum Numeric {
    RPL_WELCOME = 1,
    RPL_YOURHOST = 2,
    RPL_CREATED = 3,
    RPL_MYINFO = 4,
    RPL_BOUNCE = 5,

    RPL_ENDOFWHO = 315,

    RPL_LISTSTART = 321,
    RPL_LIST = 322,
    RPL_LISTEND = 323,

    // RPL_TOPIC = 332,
    // RPL_TOPICWHOTIME = 333,
    RPL_WHOREPLY = 352,
    RPL_NAMREPLY = 353,
    RPL_ENDOFNAMES = 366,

    ERR_NICKNAMEINUSE = 433,
    ERR_ALREADYREGISTERED = 462,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref prefix) = self.prefix {
            write!(f, ":{} ", prefix)?
        }
        write!(f, "{}", self.command)
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        match self {
            Nick(nick) => write!(f, "NICK {}", nick),
            User(user, mode, unused, realname) => {
                write!(f, "USER {} {} {} {}", user, mode, unused, realname)
            }
            Join(channels, keys, flag) => {
                if *flag {
                    write!(f, "JOIN 0")
                } else {
                    write!(
                        f,
                        "{}",
                        ["JOIN".to_string(), channels.join(","), keys.join(",")].join(" ")
                    )
                }
            }
            List(channels, target) => {
                write!(f, "LIST")?;
                if let Some(ch_str) = channels.as_ref().map(|chs| chs.join(",")) {
                    if !ch_str.is_empty() {
                        write!(f, " {}", ch_str)?;
                    }
                }
                if let Some(t) = target {
                    write!(f, " {}", t)?;
                }
                Ok(())
            }
            WHO{mask} => write!(f, "WHO {}", mask),
            Numeric(numeric, params) => write!(f, "{:03} {}", *numeric as u16, params.join(" ")),
            Invalid => write!(f, "INVALID"),
        }
    }
}
