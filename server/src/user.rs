use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use common::message::{Command, Message, Numeric};

pub struct User {
    pub stream: Arc<TcpStream>,
    pub username: String,
    pub nickname: String,
    pub hostname: String,
    pub realname: String,
}
pub type SharedUser = Arc<Mutex<User>>;

impl User {
    pub fn new(stream: TcpStream) -> Self {
        User {
            stream: Arc::new(stream),
            username: String::new(),
            nickname: String::new(),
            hostname: String::new(),
            realname: String::new(),
        }
    }

    pub fn send(&mut self, messages: &[Message]) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        for message in messages {
            write!(buffer, "{message}\r\n")?;
            println!("send > {message}");
        }
        self.stream.as_ref().write_all(&buffer)
    }

    pub fn reply(&mut self, replies: &[(Numeric, String)]) -> std::io::Result<()> {
        let mut messages = Vec::new();
        for (numeric, params) in replies {
            messages.push(Message::new(
                Some(String::from("akiRC")),
                Command::Numeric(*numeric, vec![self.nickname.to_owned(), params.to_owned()]),
            ))
        }
        self.send(&messages)
    }
}
