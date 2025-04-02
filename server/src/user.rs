use std::{
    io::Write, net::TcpStream, sync::{Arc, Mutex}
};

use common::message::{Command, Message, Numeric};


#[derive(Debug)]
pub struct User {
    stream: TcpStream,
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

// /// Wrapper to allow hashing of SharedUsers.
// pub struct SharedUserWrap(pub SharedUser);
// impl PartialEq for SharedUserWrap {
//     fn eq(&self, other: &Self) -> bool {
//         Arc::ptr_eq(&self.0, &other.0)
//     }
// }
// impl Eq for SharedUserWrap {}
// impl Hash for SharedUserWrap {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         state.write_usize(Arc::as_ptr(&self.0) as usize);
//     }
// }