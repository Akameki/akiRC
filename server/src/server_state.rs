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

    /// Outside of impl ServerState, this should only be called at most once per SharedUser.
    /// Panics if user is locked.
    pub fn register_user(&mut self, user: &SharedUser) {
        let nick = &user.try_lock().unwrap().nickname;
        assert!(!self.users.contains_key(nick));
        self.users.insert(nick.to_owned(), user.clone());
    }
    pub fn try_update_nick(&mut self, user: &SharedUser, new_nick: &str) -> bool {
        if self.users.contains_key(new_nick) {
            false
        } else {
            self.remove_user(user);
            user.try_lock().unwrap().nickname = new_nick.to_owned();
            self.register_user(user);
            true
        }
    }
    /// Outside of impl ServerState, this should only be called when cleaning up a disconnect.
    /// Panics if user is locked.
    pub fn remove_user(&mut self, user: &SharedUser) {
        self.users.remove(&user.lock().unwrap().nickname).unwrap();
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
        self.channels.insert(name.to_owned(), Channel::new(name.to_owned()));
        self.channels.get_mut(name).unwrap()
    }

    /// Send one or more messages to all connected users.
    /// Caller must release any locks on SharedUsers.
    pub fn broadcast(&mut self, messages: &[Message]) -> io::Result<()> {
        let message_refs: Vec<&Message> = messages.iter().collect();
        for user in self.users.values() {
            user.lock().unwrap().send(&message_refs)?;
        }
        Ok(())
    }

    // functions for debugging

    pub fn print_users(&self) {
        println!("current users: ");
        self.users.keys().for_each(|n| println!("  - {n}"));
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
