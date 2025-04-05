use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex, Weak},
};

use common::message::{Command, Message, Numeric};
use tokio::sync::mpsc;

use crate::channel::{Channel, SharedChannel};
#[derive(Debug, Clone)]
struct WeakChannel(Weak<Channel>);
pub struct User {
    // stream: TcpStream,
    tx: mpsc::Sender<Arc<Message>>,
    nickname: Mutex<String>,
    pub username: String,
    pub hostname: String,
    pub realname: String,
    channels: Mutex<HashSet<WeakChannel>>,
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
            channels: Mutex::new(HashSet::new()),
        }
    }

    pub fn get_nickname(&self) -> String {
        self.nickname.lock().unwrap().to_owned()
    }
    pub fn set_nickname(&self, nick: &str) {
        *self.nickname.lock().unwrap() = nick.to_owned();
    }

    /// Snapshot of channels that this user is in.
    pub fn get_channels(&self) -> impl Iterator<Item = SharedChannel> {
        self.channels
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .map(|channel| channel.0.upgrade().unwrap())
    }
    pub fn get_channel_names(&self) -> impl Iterator<Item = String> {
        self.get_channels().map(|channel| channel.name.clone())
    }

    pub fn _join_channel(&self, channel: &SharedChannel) -> bool {
        self.channels.lock().unwrap().insert(WeakChannel(Arc::downgrade(channel)))
    }
    pub fn _leave_channel(&self, channel: &SharedChannel) -> bool {
        self.channels.lock().unwrap().remove(&WeakChannel(Arc::downgrade(channel)))
    }

    /// nick!user@host
    pub fn get_fqn_string(&self) -> String {
        format!("{}!{}@{}", self.get_nickname(), self.username, self.hostname)
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

    pub async fn broadcast(&self, message: Arc<Message>) {
        for channel in self.get_channels() {
            channel.broadcast(Arc::clone(&message)).await;
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} | {})",
            self.get_nickname(),
            self.get_channel_names().collect::<Vec<_>>().join(", ")
        )
    }
}
impl Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}!{}@{}({}) {}]",
            self.get_nickname(),
            self.username,
            self.hostname,
            self.realname,
            self.get_channel_names().collect::<Vec<_>>().join(", ")
        )
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

impl PartialEq for WeakChannel {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WeakChannel {}
impl Hash for WeakChannel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Weak::as_ptr(&self.0).hash(state);
    }
}
