use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Message {
    // tags
    pub prefix: Option<String>,
    pub command: Command,
}

// optional parameters are *not* of type Option to prevent ambiguity with empty Strings and Vecs,
// except for commands like TOPIC where presence of an empty argument has semantic meaning.

#[rustfmt::skip]
#[derive(Debug, PartialEq)]
pub enum Command {
    /* Connection Messages */
    // CAP
    // AUTHENTICATE
    // PASS
    NICK { nickname: String },
    /// "" is used as a sentinal value for invalid usernames.
    USER { username: String, _1: (), _2: (), realname: String },
    PING { token: String },
    PONG { server: String, token: String },
    // OPER
    QUIT { reason: String },
    ERROR { reason: String },

    /* Channel Operations */
    JOIN { channels: Vec<String>, keys: Vec<String>, alt: bool },
    PART { channels: Vec<String>, reason: String },
    TOPIC { channel: String, topic: Option<String> },
    // NAMES
    LIST { channels: Vec<String>, elistconds: Option<String> },
    // INVITE
    // KICK

    /* Server Queries and Commands */
    MOTD { target: String },
    // VERSION
    // ADMIN
    // CONNECT
    // LUSERS
    // TIME
    // STATS
    // HELP
    // INFO
    MODE { target: String, modestring: String, modeargs: Vec<String> },

    /* Sending Messages */
    PRIVMSG { targets: Vec<String>, text: String },
    // NOTICE

    /* User Based Queries */
    WHO { mask: String },
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

    /// Command name, Numeric reply, Numeric params
    /// Unknown command or command with invalid params.
    /// If the second value is Some, the user can be replied to with the Numeric and its params.
    Invalid(String, Option<Numeric>, String),
    /// Contains the command and params for the server to send directly.
    Raw(String),
}

impl Message {
    pub fn new(prefix: Option<&str>, command: Command) -> Self {
        Message { prefix: prefix.map(|s| s.to_string()), command }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum Numeric {
    // Client-Server Connections 001~099
    RPL_WELCOME = 1,
    RPL_YOURHOST = 2,
    RPL_CREATED = 3,
    RPL_MYINFO = 4,
    RPL_ISUPPORT = 5,
    // Command Replies 200 ~ 399
    RPL_UMODEIS = 221,
    RPL_ENDOFWHO = 315,
    RPL_LISTSTART = 321,
    RPL_LIST = 322,
    RPL_LISTEND = 323,
    RPL_CHANNELMODEIS = 324,
    RPL_CREATIONTIME = 329,
    RPL_NOTOPIC = 331,
    RPL_TOPIC = 332,
    RPL_TOPICWHOTIME = 333,
    RPL_WHOREPLY = 352,
    RPL_NAMREPLY = 353,
    RPL_ENDOFNAMES = 366,
    RPL_MOTD = 372,
    RPL_MOTDSTART = 375,
    RPL_ENDOFMOTD = 376,

    // Error Replies 400~509
    ERR_NOSUCHNICK = 401,
    ERR_NOSUCHSERVER = 402,
    ERR_NOSUCHCHANNEL = 403,
    ERR_NORECIPIENT = 411,
    ERR_NOTEXTTOSEND = 412,
    ERR_UNKNOWNCOMMAND = 421,
    ERR_NOMOTD = 422,
    ERR_NONICKNAMEGIVEN = 431,
    ERR_ERRONEUSNICKNAME = 432,
    ERR_NICKNAMEINUSE = 433,
    ERR_NOTONCHANNEL = 442,
    ERR_NEEDMOREPARAMS = 461,
    ERR_ALREADYREGISTERED = 462,
    ERR_UNKNOWNMODE = 472,
    ERR_CHANOPRIVSNEEDED = 482,

    ERR_UMODEUNKNOWNFLAG = 501,
    ERR_USERSDONTMATCH = 502,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref prefix) = self.prefix {
            write!(f, ":{} ", prefix)?
        }
        write!(f, "{}", self.command)
    }
}

// Used for message assembly. Should be refactored to minimize duplicated logic.
impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;

        match self {
            /* Connection Messages */
            // CAP
            // AUTHENTICATE
            // PASS
            NICK { nickname } => write!(f, "NICK {}", nickname),
            USER { username, _1, _2, realname } => {
                write!(f, "USER {} 0 * {}", username, realname)
            }
            PING { token } => write!(f, "PING :{}", token),
            PONG { server, token } => {
                write!(f, "PONG")?;
                if !server.is_empty() {
                    write!(f, " {}", server)?;
                }
                write!(f, " :{}", token)
            }
            // OPER
            QUIT { reason } => {
                if !reason.is_empty() {
                    write!(f, "QUIT :{}", reason)
                } else {
                    write!(f, "QUIT")
                }
            }
            ERROR { reason } => {
                write!(f, "ERROR :{}", reason)
            }

            /* Channel Operations */
            JOIN { channels, keys, alt } => {
                if *alt {
                    write!(f, "JOIN 0")
                } else {
                    write!(f, "JOIN {}", channels.join(","))?;
                    if !keys.is_empty() {
                        write!(f, " {}", keys.join(","))?;
                    }
                    Ok(())
                }
            }
            PART { channels, reason } => {
                write!(f, "PART {}", channels.join(","))?;
                if !reason.is_empty() {
                    write!(f, " :{}", reason)?;
                }
                Ok(())
            }
            TOPIC { channel, topic } => {
                write!(f, "TOPIC {}", channel)?;
                if let Some(t) = topic {
                    write!(f, " :{}", t)?;
                }
                Ok(())
            }
            // NAMES
            LIST { channels, elistconds } => {
                write!(f, "LIST")?;
                if !channels.is_empty() {
                    write!(f, " {}", channels.join(","))?;
                }
                if let Some(t) = elistconds {
                    write!(f, " {}", t)?;
                }
                Ok(())
            }
            // INVITE
            // KICK

            /* Server Queries and Commands */
            MOTD { target } => {
                write!(f, "MOTD")?;
                if !target.is_empty() {
                    write!(f, " {}", target)?;
                }
                Ok(())
            }
            // VERSION
            // ADMIN
            // CONNECT
            // LUSERS
            // TIME
            // STATS
            // HELP
            // INFO
            MODE { target, modestring, modeargs } => {
                write!(f, "MODE {}", target)?;
                if !modestring.is_empty() {
                    write!(f, " {}", modestring)?;
                }
                if !modeargs.is_empty() {
                    write!(f, " {}", modeargs.join(","))?;
                }
                Ok(())
            }

            /* Sending Messages */
            PRIVMSG { targets, text } => write!(f, "PRIVMSG {} :{}", targets.join(","), text),
            // NOTICE

            /* User Based Queries */
            WHO { mask } => write!(f, "WHO {}", mask),
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
            Numeric(numeric, params) => write!(f, "{:03} {}", *numeric as u16, params.join(" ")),
            Invalid(name, num, str) => {
                write!(f, "Invalid(\"{}\", {} <client> {})", name, num.map_or(0, |n| n as u16), str)
            }
            Raw(str) => write!(f, "{str}"),
        }
    }
}
