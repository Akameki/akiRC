use std::{
    io::Write,
    net::TcpStream,
    str,
    sync::{Arc, Mutex},
};

use common::{Message, Numeric};

pub struct User {
    pub stream: TcpStream,
    pub username: String,
    pub nickname: String,
    pub hostname: String,
    pub realname: String,
}
pub type SharedUser = Arc<Mutex<User>>;

impl User {
    pub fn new(stream: TcpStream) -> Self {
        User {
            stream,
            username: String::from(""),
            nickname: String::from(""),
            hostname: String::from(""),
            realname: String::from(""),
        }
    }

    pub fn create_reply(&self, num: Numeric, params: &str) -> Message {
        Message::new_numeric("akiRC", num, &self.nickname, params)
    }

    pub fn send(&mut self, messages: &[Message]) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        for message in messages {
            write!(buffer, "{message}\r\n")?;
            println!("send > {message}");
        }
        self.stream.write_all(&buffer)
    }
}
