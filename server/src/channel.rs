use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex, Weak},
};

use common::message::Message;

use crate::user::{SharedUser, User};

#[derive(Clone)]
struct WeakUser(Weak<User>);
pub struct Channel {
    pub name: String,
    pub topic: String,
    users: Mutex<HashSet<WeakUser>>,
}
pub type SharedChannel = Arc<Channel>;

impl Channel {
    pub fn new(name: String) -> Self {
        Channel { name, users: Mutex::new(HashSet::new()), topic: String::new() }
    }

    /// Snapshot of users in this channel.
    pub fn get_users(&self) -> impl Iterator<Item = SharedUser> {
        self.users.lock().unwrap().clone().into_iter().map(|user| user.0.upgrade().unwrap())
    }
    pub fn get_nicks(&self) -> impl Iterator<Item = String> {
        self.get_users().map(|user| user.get_nickname())
    }
    pub fn contains_user(&self, user: &SharedUser) -> bool {
        self.users.lock().unwrap().contains(&WeakUser(Arc::downgrade(user)))
    }
    pub fn user_count(&self) -> usize {
        self.users.lock().unwrap().len()
    }

    pub fn _add_user(&self, user: &SharedUser) -> bool {
        self.users.lock().unwrap().insert(WeakUser(Arc::downgrade(user)))
    }
    pub fn _remove_user(&self, user: &SharedUser) -> bool {
        self.users.lock().unwrap().remove(&WeakUser(Arc::downgrade(user)))
    }

    pub async fn broadcast(&self, message: Arc<Message>) {
        for user in self.get_users() {
            user.send(Arc::clone(&message)).await;
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} | {})", self.name, self.get_nicks().collect::<Vec<_>>().join(", "))
    }
}

impl Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} ({}) {}]",
            self.name,
            self.topic,
            self.get_nicks().collect::<Vec<_>>().join(", ")
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
