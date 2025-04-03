use std::sync::{Arc, Mutex};

use common::message::{Command, Message, Numeric};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct User {
    // stream: TcpStream,
    tx: mpsc::Sender<Arc<Message>>,
    nickname: Mutex<String>,
    pub username: String,
    pub hostname: String,
    pub realname: String,
    // pub channels: HashSet<Weak<Mutex<Channel>>>,
}
/// MUST hold a lock on SharedServerState before any SharedUser can be locked.
pub type SharedUser = Arc<User>;

impl User {
    pub fn new(tx: mpsc::Sender<Arc<Message>>, hostname: String) -> Self {
        User {
            tx,
            nickname: Mutex::new(String::new()),
            username: String::new(),
            hostname,
            realname: String::new(),
            // channels: HashSet::new(),
        }
    }

    pub fn get_nickname(&self) -> String {
        self.nickname.lock().unwrap().to_owned()
    }
    pub fn set_nickname(&self, nick: &str) {
        *self.nickname.lock().unwrap() = nick.to_owned();
    }

    /// nickname!user@host
    pub fn target_str(&self) -> String {
        format!(
            "{}!{}@{}",
            self.get_nickname(),
            self.username,
            self.hostname
        )
    }

    pub async fn send(&self, message: Arc<Message>) {
        let _ = self.tx.send(message).await;
    }

    /// Write all parameters after the target as one string, including the trailing ":".
    /// It will all be represented as one parameter, though it should not matter for writing.
    pub async fn reply(&self, numeric: Numeric, params: &str) {
        self.send(Arc::new(Message::new(
            Some("akiRC"),
            Command::Numeric(numeric, vec![self.get_nickname(), params.to_owned()]),
        )))
        .await;
    }
}

// #[macro_export]
// /// Used for User::reply_multiple(). List each reply in this format:
// /// `Numeric => format_string, ...args;`
// macro_rules! replies {
//     ($( $num:expr => $fmt:literal $(, $args:expr )* );* $(;)?) => {
//         &[$(($num, format!($fmt, $($args),*))),*]
//     };
// }
