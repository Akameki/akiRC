use std::{
    collections::HashMap, io, sync::{Arc, Mutex}
};

use common::message::Message;

use crate::{channel::Channel, user::SharedUser};

pub struct ServerState {
    pub users: HashMap<String, SharedUser>, // key=nick
    pub channels: HashMap<String, Channel>, // key=name
}
pub type SharedServerState = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            users: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn remove_user(&mut self, user: &SharedUser) -> Option<SharedUser> {
        let nick = &user.lock().unwrap().nickname;

        if let Some(stored_user) = self.users.get(nick) {
            if Arc::ptr_eq(user, stored_user) { // sanity check
                return self.users.remove(nick);
            } else {
                eprintln!("{user:?}'s nick {nick} maps to a different User in ServerState: {stored_user:?}");
            }
        }
        None
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
