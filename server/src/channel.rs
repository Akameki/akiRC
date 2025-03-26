use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use crate::user::SharedUser;
pub struct Channel {
    pub name: String,
    pub users: HashSet<SharedUser>,
    pub topic: String,
}

pub type SharedChannel = Arc<Mutex<Channel>>;
