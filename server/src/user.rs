use std::sync::{Arc, Mutex};

pub struct User {
    pub username: String,
    pub nickname: String,
    pub hostname: String,
    pub realname: String,
    pub registered: bool,
}

pub type SharedUser = Arc<Mutex<User>>;
