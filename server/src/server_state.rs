use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    sync::Arc,
};

use common::message::Message;
use tokio::sync::Mutex; // todo: avoid tokio Mutex?

use crate::{
    channel::{Channel, SharedChannel},
    user::{SharedUser, User},
};

pub struct ServerState {
    users: HashMap<String, SharedUser>,       // key=nick
    channels: HashMap<String, SharedChannel>, // key=name
    unregistered_nicks: HashSet<String>,
}
pub type SharedServerState = Arc<Mutex<ServerState>>;

// functions panic if a SharedUser that requires locking is already locked.
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            users: HashMap::new(),
            channels: HashMap::new(),
            unregistered_nicks: HashSet::new(),
        }
    }

    pub fn contains_nick(&self, nick: &str) -> bool {
        self.users.contains_key(nick)
    }
    pub fn get_user(&self, nick: &str) -> Option<SharedUser> {
        self.users.get(nick).map(Arc::clone)
    }
    pub fn users(&self) -> impl Iterator<Item = SharedUser> {
        self.users.values().map(Arc::clone)
    }
    pub fn channels(&self) -> impl Iterator<Item = SharedChannel> {
        self.channels.values().map(Arc::clone)
    }

    pub fn try_update_nick(&mut self, user: &SharedUser, new_nick: &str) -> bool {
        if self.contains_nick(new_nick) || self.unregistered_nicks.contains(new_nick) {
            return false;
        }
        let user2 = self.users.remove(&user.get_nickname()).unwrap();
        user2.set_nickname(new_nick);
        self.users.insert(new_nick.to_owned(), user2);
        true
    }

    /// Only use in main.rs
    pub fn try_update_unregistered_nick(&mut self, nick: &str, new_nick: &str) -> bool {
        if self.contains_nick(new_nick) || self.unregistered_nicks.contains(new_nick) {
            return false;
        }
        if !nick.is_empty() {
            assert!(self.unregistered_nicks.remove(nick));
        }
        self.unregistered_nicks.insert(new_nick.to_owned())
    }
    /// Only use in main.rs
    pub fn register_user(&mut self, user: User) -> SharedUser {
        let nick = user.get_nickname();
        let registered_user = Arc::new(user);
        assert!(self.unregistered_nicks.remove(&nick));
        assert!(self.users.insert(nick, Arc::clone(&registered_user)).is_none());
        registered_user
    }
    /// Only use in main.rs
    pub fn remove_unregistered_nick(&mut self, user: User) {
        let nick = user.get_nickname();
        self.unregistered_nicks.remove(&nick);
    }
    /// Only use in main.rs
    pub fn remove_user(&mut self, user: SharedUser) {
        let nick = user.get_nickname();
        for channel in user.get_channels() {
            self.remove_user_from_channel(&user, &channel);
        }
        assert!(Arc::ptr_eq(&user, &self.users.remove(&nick).unwrap()));
    }

    pub fn get_channel_names(&self) -> impl Iterator<Item = String> {
        self.channels.keys().cloned()
    }
    pub fn get_channel(&self, name: &str) -> Option<SharedChannel> {
        self.channels.get(name).map(Arc::clone)
    }
    pub fn get_channels(&self) -> impl Iterator<Item = SharedChannel> {
        self.channels.values().map(Arc::clone)
    }
    pub fn contains_channel_name(&self, name: &str) -> bool {
        self.channels.contains_key(name)
    }
    /// Returns &mut to new Channel. Panics if channel already exists.
    pub fn create_channel(&mut self, name: &str) -> SharedChannel {
        assert!(!self.channels.contains_key(name));
        self.channels.insert(name.to_owned(), Arc::new(Channel::new(name.to_owned())));
        Arc::clone(self.channels.get(name).unwrap())
    }

    pub fn add_user_to_channel(&mut self, user: &SharedUser, channel: &SharedChannel) -> bool {
        let (r1, r2) = (user._join_channel(channel), channel._add_user(user));
        assert_eq!(r1, r2);
        r1
    }
    pub fn remove_user_from_channel(&mut self, user: &SharedUser, channel: &SharedChannel) -> bool {
        let (r1, r2) = (user._leave_channel(channel), channel._remove_user(user));
        assert_eq!(r1, r2);
        if channel.user_count() == 0 {
            self.channels.remove(&channel.name);
        }
        r1
    }

    pub async fn broadcast(&mut self, message: Arc<Message>) {
        for user in self.users.values() {
            user.send(Arc::clone(&message)).await;
        }
    }
}

impl Display for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Users ({}): {}",
            self.users.len(),
            self.users().map(|u| u.to_string()).collect::<Vec<_>>().join(", ")
        )?;
        write!(
            f,
            "Channels ({}): {}",
            self.channels.len(),
            self.channels().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}
impl Debug for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-- {} Users, {} Channels --", self.users.len(), self.channels.len())?;
        writeln!(f, "Users: {:?}", self.users().collect::<Vec<_>>())?;
        write!(f, "Channels: {:?}", self.channels().collect::<Vec<_>>())
    }
}
