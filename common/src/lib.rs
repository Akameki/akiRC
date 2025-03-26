pub mod parse;

use std::fmt::Display;

// pub type IRCPrefix = String;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Numeric {
    RPL_WELCOME = 1,
    RPL_YOURHOST = 2,
    RPL_CREATED = 3,
    RPL_MYINFO = 4,
    RPL_BOUNCE = 5,
}

#[derive(Debug)]
pub enum Command {
    // CAP(Option<String>, String, Option<String>, Option<String>), // [*] <subcommand> [*] [<params>]
    /// `<nickname>``
    Nick(String),
    /// `<user> <mode> <unused> <realname>`
    User(String, String, String, String),

    Numeric(Numeric, Vec<String>),

    Invalid(),
}

impl Command {
    fn new(command: &str, params: Vec<String>) -> Self {
        use Command::*;

        match command {
            // "CAP" => Command::CAP(params[0], params[1], params[2], params[3])
            "NICK" => Nick(params[0].clone()),
            "USER" => User(
                params[0].clone(),
                params[1].clone(),
                params[2].clone(),
                params[3].clone(),
            ),
            _ => Invalid(),
        }
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
            Numeric(numeric, params) => write!(f, "{:03} {}", *numeric as u16, params.join(" ")),
            Invalid() => write!(f, "INVALID"),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: Command,
}

impl Message {
    pub fn new(prefix: Option<String>, command: Command) -> Self {
        Message { prefix, command }
    }

    pub fn new_numeric(prefix: &str, numeric: Numeric, target: &str, params: &str) -> Message {
        Self::new(
            Some(prefix.into()),
            Command::Numeric(numeric, vec![target.into(), params.into()]),
        )
    }
}
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref prefix) = self.prefix {
            write!(f, ":{} ", prefix)?
        }
        write!(f, "{}", self.command)
    }
}
