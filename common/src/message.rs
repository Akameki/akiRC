use std::fmt::Display;

#[derive(Debug)]
pub struct Message {
    // tags
    pub prefix: Option<String>,
    pub command: Command,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    // CAP
    /// `<nickname>``
    Nick(String),
    /// `<user> <mode> <unused> <realname>`
    User(String, String, String, String),
    /// `[<channels> [target]]`
    List(Option<Vec<String>>, Option<String>),

    Numeric(Numeric, Vec<String>),

    Invalid,
}

impl Message {
    pub fn new(prefix: Option<String>, command: Command) -> Self {
        Message { prefix, command }
    }
}

impl Command {
    pub fn new(command: &str, params: &[&str]) -> Self {
        use Command::*;

        let len = params.len();
        let mut params_iter = params.iter().cloned();

        macro_rules! req {
            () => {
                params_iter.next().unwrap().to_owned()
            };
        }

        match command {
            "NICK" => Nick(String::from(params[0])),
            "USER" if len >= 4 => User(req!(), req!(), req!(), req!()),
            _ => Invalid,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum Numeric {
    RPL_WELCOME = 1,
    RPL_YOURHOST = 2,
    RPL_CREATED = 3,
    RPL_MYINFO = 4,
    RPL_BOUNCE = 5,

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
            Numeric(numeric, params) => write!(f, "{:03} {}", *numeric as u16, params.join(" ")),
            Invalid => write!(f, "INVALID"),
        }
    }
}
