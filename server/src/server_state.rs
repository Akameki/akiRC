use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use common::message::Message;

use crate::{channel::Channel, user::SharedUser};

pub struct ServerState {
    users: HashMap<String, SharedUser>, // key=nick
    channels: HashMap<String, Channel>, // key=name
}
pub type SharedServerState = Arc<Mutex<ServerState>>;

// functions panic if a SharedUser that requires locking is already locked.
impl ServerState {
    pub fn new() -> Self {
        ServerState {
            users: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn contains_nick(&self, nick: &str) -> bool {
        self.users.contains_key(nick)
    }

    /// Only use in main.rs
    pub fn insert_user(&mut self, nick: &str, user: &SharedUser) {
        assert!(!self.contains_nick(nick));
        self.users.insert(nick.to_owned(), user.clone());
    }
    pub fn try_update_nick(&mut self, user: &SharedUser, new_nick: &str) -> bool {
        if self.contains_nick(new_nick) {
            return false;
        }
        self.remove_user(user);
        user.try_lock().unwrap().nickname = new_nick.to_owned();
        self.insert_user(new_nick, user);
        true
    }
    /// Only use in main.rs
    pub fn remove_user(&mut self, user: &SharedUser) {
        if !user.try_lock().unwrap().nickname.is_empty() {
            self.users.remove(&user.try_lock().unwrap().nickname).unwrap();
            // todo: remove from channels
        }
    }
    pub fn get_channel(&mut self, name: &str) -> Option<&mut Channel> {
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
        self.channels
            .insert(name.to_owned(), Channel::new(name.to_owned()));
        self.channels.get_mut(name).unwrap()
    }

    /// Send one or more messages to all connected users.
    /// Caller must release any locks on SharedUsers.
    pub fn broadcast(&mut self, messages: &[Message]) -> io::Result<()> {
        let message_refs: Vec<&Message> = messages.iter().collect();
        for user in self.users.values() {
            let user = user.try_lock().unwrap();
            if user.registered {
                user.send(&message_refs)?;
            }
        }
        Ok(())
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
