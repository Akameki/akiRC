use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use common::message::{Command, Message, Numeric};

#[derive(Debug)]
pub struct User {
    stream: TcpStream,
    pub registered: bool,
    pub username: String,
    pub nickname: String,
    pub hostname: String,
    pub realname: String,
    // pub channels: HashSet<Weak<Mutex<Channel>>>,
}
/// MUST hold a lock on SharedServerState before any SharedUser can be locked.
pub type SharedUser = Arc<Mutex<User>>;

impl User {
    pub fn new(stream: TcpStream) -> Self {
        User {
            stream,
            registered: false,
            username: String::new(),
            nickname: String::new(),
            hostname: String::new(),
            realname: String::new(),
            // channels: HashSet::new(),
        }
    }

    /// nickname!user@host
    pub fn target_str(&self) -> String {
        format!("{}!{}@{}", self.nickname, self.username, self.hostname)
    }

    pub fn send(&self, messages: &[&Message]) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        for message in messages {
            write!(buffer, "{message}\r\n")?;
            println!("send > {message}");
        }
        (&self.stream).write_all(&buffer)
    }

    /// Write all parameters after the target as one string, including the trailing ":".
    /// It will all be represented as one parameter, though it should not matter for writing.
    pub fn reply(&self, numeric: Numeric, params: &str) -> std::io::Result<()> {
        self.reply_multiple(&[(numeric, params.to_owned())])
    }
    pub fn reply_multiple(&self, replies: &[(Numeric, String)]) -> std::io::Result<()> {
        let mut messages = Vec::new();
        for (numeric, params) in replies {
            messages.push(Message::new(
                Some("akiRC"),
                Command::Numeric(*numeric, vec![self.nickname.to_owned(), params.to_string()]),
            ))
        }
        let message_refs: Vec<&Message> = messages.iter().collect();
        self.send(&message_refs)
    }
}

#[macro_export]
/// Used for User::reply_multiple(). List each reply in this format:  
/// `Numeric => format_string, ...args;`
macro_rules! replies {
    ($( $num:expr => $fmt:literal $(, $args:expr )* );* $(;)?) => {
        &[$(($num, format!($fmt, $($args),*))),*]
    };
}
