use std::{
    char,
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex, Weak},
};

use common::message::{Command, Message, Numeric};
use tokio::sync::mpsc;

use crate::channel::{SharedChannel, WeakChannel};
pub struct User {
    tx: mpsc::Sender<Arc<Message>>,
    nickname: Mutex<String>,
    pub username: String,
    pub hostname: String,
    pub realname: String,
    channels: Mutex<HashSet<WeakChannel>>,
    modes: Mutex<HashSet<char>>,
}
#[derive(Clone)]
pub struct WeakUser(pub Weak<User>);
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
            modes: Mutex::new(HashSet::new()),
        }
    }
    pub fn are_same(user1: &SharedUser, user2: &SharedUser) -> bool {
        Arc::ptr_eq(user1, user2)
    }

    /* Nick */
    pub fn get_nickname(&self) -> String {
        self.nickname.lock().unwrap().to_owned()
    }
    pub fn set_nickname(&self, nick: &str) {
        *self.nickname.lock().unwrap() = nick.to_owned();
    }
    /// nick!user@host
    pub fn get_fqn_string(&self) -> String {
        format!("{}!{}@{}", self.get_nickname(), self.username, self.hostname)
    }

    /* Channels */
    /// Snapshot of channels that this user is in.
    pub fn get_channels(&self) -> impl Iterator<Item = SharedChannel> {
        self.channels
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .map(|channel| channel.0.upgrade().unwrap())
    }
    pub fn is_in_channel(&self, channel: &SharedChannel) -> bool {
        self.channels.lock().unwrap().contains(&WeakChannel(Arc::downgrade(channel)))
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

    /* Modes */
    pub fn get_modes(&self) -> impl Iterator<Item = char> {
        self.modes.lock().unwrap().clone().into_iter()
    }
    pub fn add_mode(&self, mode: char) -> bool {
        self.modes.lock().unwrap().insert(mode)
    }
    pub fn remove_mode(&self, mode: char) -> bool {
        self.modes.lock().unwrap().remove(&mode)
    }

    /* Messaging */
    pub async fn send(&self, message: Arc<Message>) {
        let _ = self.tx.send(message).await;
    }
    /// Write all parameters after the target as one string, including the trailing ":".
    /// It will all be represented as one parameter, though it should not matter for writing.
    pub async fn reply(&self, numeric: Numeric, params: &str) {
        self.send(Arc::new(Message::new(
            Some("akiRC.chat"), // FIXME: hardcoded servername
            Command::Numeric(numeric, vec![self.get_nickname(), params.to_owned()]),
        )))
        .await;
    }
    pub async fn broadcast(&self, message: Arc<Message>) {
        let mut seen = HashSet::new();
        for channel in self.get_channels() {
            for user in channel.get_users() {
                if seen.insert(WeakUser(Arc::downgrade(&user))) {
                    user.send(Arc::clone(&message)).await;
                }
            }
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

impl PartialEq for WeakUser {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WeakUser {}
impl Hash for WeakUser {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}
