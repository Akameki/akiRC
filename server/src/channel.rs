use std::{
    collections::HashSet, hash::Hash, io, sync::{Arc, Mutex, Weak}
};

use common::message::Message;

use crate::user::{SharedUser, User};

struct WeakMutexUser(Weak<Mutex<User>>);
pub struct Channel {
    pub name: String,
    pub topic: String,
    users: HashSet<WeakMutexUser>,
}

// pub type SharedChannel = Arc<Mutex<Channel>>;

impl Channel {
    pub fn new(name: String) -> Self {
        Channel {
            name,
            users: HashSet::new(),
            topic: String::new(),
        }
    }

    pub fn get_user_nicks(&self) -> Vec<String> {
        self.users
            .iter()
            .filter_map(|user| user.0.upgrade())
            .map(|user| user.lock().unwrap().nickname.clone())
            .collect()
    }
    pub fn get_users(&self) -> Vec<SharedUser> {
        self.users
            .iter()
            .filter_map(|user| user.0.upgrade())
            .collect()
    }
    pub fn user_count(&self) -> usize {
        // todo: should we worry about filtering dropped references if threads manually remove themselves?
        // or also can encapsulate clean-up into ServerState.
        self.users.len()
    }
    
    pub fn add_user(&mut self, user: &SharedUser) -> bool {
        self.users.insert(WeakMutexUser(Arc::downgrade(user)))
    }
    pub fn remove_user(&mut self, user: &SharedUser) -> bool {
        self.users.remove(&WeakMutexUser(Arc::downgrade(user)))
    }

    pub fn broadcast(&self, message: &Message) -> io::Result<()> {
        for user in self.users.iter() {
            if let Some(user) = user.0.upgrade() {
                user.lock().unwrap().send(&[message])?;
            }
        }
        Ok(())
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new("".to_owned())
    }
}

impl PartialEq for WeakMutexUser {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WeakMutexUser {}
impl Hash for WeakMutexUser {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}