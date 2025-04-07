use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex, Weak},
    time::SystemTime,
};

use common::message::Message;

use crate::user::{SharedUser, WeakUser};

/// Each mode is one of four types, as specified by IRCv3 docs.  
/// ChannelModes only stores modes, and Channel provides no checks for privaleges.  
/// I.e. Channel and ChannelModes only sees arbitrary letters. Implementation must be elsewhere.
#[derive(Clone)]
pub struct ChannelModes {
    /* Type A: list modes */
    /* Type B: param on set */
    /* Type C: param always */
    /* Type D: no params */
    /// secret channel
    pub s: bool,
}

pub struct Channel {
    pub creation_time: String,
    pub name: String,
    /// topic, who, time
    topic_info: Mutex<Option<(String, String, String)>>,
    users: Mutex<HashSet<WeakUser>>,
    modes: Mutex<ChannelModes>,
}
#[derive(Clone)]
pub struct WeakChannel(pub Weak<Channel>);
pub type SharedChannel = Arc<Channel>;

impl Channel {
    pub fn new(name: String) -> Self {
        Channel {
            creation_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            name,
            users: Mutex::new(HashSet::new()),
            topic_info: Mutex::new(None),
            modes: Mutex::new(ChannelModes { s: false }),
        }
    }

    /// topic, who, time
    pub fn get_topic_info(&self) -> Option<(String, String, String)> {
        self.topic_info.lock().unwrap().clone()
    }
    pub fn set_topic(&self, user: &SharedUser, topic: &str) {
        *self.topic_info.lock().unwrap() = Some((
            topic.to_string(),
            user.get_fqn_string(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string(),
        ));
    }

    /* Users */
    pub fn contains_user(&self, user: &SharedUser) -> bool {
        self.users.lock().unwrap().contains(&WeakUser(Arc::downgrade(user)))
    }
    pub fn user_count(&self) -> usize {
        self.users.lock().unwrap().len()
    }
    /// Snapshot of users in this channel.
    pub fn get_users(&self) -> impl Iterator<Item = SharedUser> {
        self.users.lock().unwrap().clone().into_iter().map(|user| user.0.upgrade().unwrap())
    }
    /// Snapshot of nicks in this channel.
    pub fn get_nicks(&self) -> impl Iterator<Item = String> {
        self.get_users().map(|user| user.get_nickname())
    }
    pub fn _add_user(&self, user: &SharedUser) -> bool {
        self.users.lock().unwrap().insert(WeakUser(Arc::downgrade(user)))
    }
    pub fn _remove_user(&self, user: &SharedUser) -> bool {
        self.users.lock().unwrap().remove(&WeakUser(Arc::downgrade(user)))
    }

    /* Modes */
    /// Snapshot of channel modes
    pub fn get_modes(&self) -> ChannelModes {
        self.modes.lock().unwrap().clone()
    }
    pub fn set_mode_type_d(&self, mode: char, value: bool) -> bool {
        let mut modes = self.modes.lock().unwrap();
        let flag = match mode {
            's' => &mut modes.s,
            _ => panic!("mode was not checked first!"),
        };
        if *flag == value {
            false
        } else {
            *flag = value;
            true
        }
    }

    /* Messaging */
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
        write!(f, "[{} {}]", self.name, self.get_nicks().collect::<Vec<_>>().join(", "),)
    }
}

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
