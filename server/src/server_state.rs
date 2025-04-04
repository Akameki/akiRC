use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use common::message::Message;
use tokio::sync::Mutex;

use crate::{
    channel::Channel,
    user::{SharedUser, User},
};

pub struct ServerState {
    users: HashMap<String, SharedUser>, // key=nick
    channels: HashMap<String, Channel>, // key=name
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
    pub fn get_user(&self, nick: &str) -> Option<&SharedUser> {
        self.users.get(nick)
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
        // todo: remove from channels
        assert!(Arc::ptr_eq(&user, &self.users.remove(&nick).unwrap()));
    }

    pub fn get_channel_names(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }
    pub fn get_channel(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }
    pub fn get_channel_mut(&mut self, name: &str) -> Option<&mut Channel> {
        self.channels.get_mut(name)
    }
    // todo: rename?
    pub fn get_channels(&self) -> impl Iterator<Item = &Channel> {
        self.channels.values()
    }
    pub fn contains_channel(&self, name: &str) -> bool {
        self.channels.contains_key(name)
    }
    /// Returns &mut to new Channel. Panics if channel already exists.
    pub fn create_channel(&mut self, name: &str) -> &mut Channel {
        assert!(!self.channels.contains_key(name));
        self.channels.insert(name.to_owned(), Channel::new(name.to_owned()));
        self.channels.get_mut(name).unwrap()
    }

    pub async fn broadcast(&mut self, message: Arc<Message>) {
        for user in self.users.values() {
            user.send(Arc::clone(&message)).await;
        }
    }
}
