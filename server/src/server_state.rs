use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{channel::SharedChannel, user::SharedUser};

pub struct ServerState {
    pub users: Arc<Mutex<HashMap<String, SharedUser>>>, // nick as key
    pub channels: Arc<Mutex<HashMap<String, SharedChannel>>>, // name as key
}
pub type SharedServerState = Arc<Mutex<ServerState>>;

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            users: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn remove_user(&mut self, user: &SharedUser) -> Option<SharedUser> {
        let mut users_locked = self.users.lock().unwrap();
        let nick = &user.lock().unwrap().nickname;

        if let Some(stored_user) = users_locked.get(nick) {
            if Arc::ptr_eq(user, stored_user) {
                return users_locked.remove(nick);
            }
        }
        None
    }

    pub fn print_users(&self) {
        let users = self.users.lock().unwrap();
        println!("current users: ");
        users.keys().for_each(|n| println!("  - {n}"));
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
