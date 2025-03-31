use std::io;

use common::{
    IrcError,
    message::{Command, Message, Numeric::*},
};

use crate::{server_state::SharedServerState, user::SharedUser};

/// Message handling AFTER connection registration/handshake
pub fn handle_message(
    server: &SharedServerState,
    user: &SharedUser,
    message: Message,
) -> Result<(), IrcError> {
    use Command::*;

    match message.command {
        Nick(nick) => handle_nick(server, user, nick),
        User(..) => Ok(user.lock().unwrap().reply(&[(
            ERR_ALREADYREGISTERED,
            ":Unauthorized command (already registered)".to_string(),
        )])?),
        List(channels, target) => handle_list(server, user, channels, target),

        Numeric(numeric, items) => todo!(),
        Invalid => todo!(),
    }
}

fn handle_nick(
    server: &SharedServerState,
    user: &SharedUser,
    nick: String,
) -> Result<(), IrcError> {
    let mut server = server.lock().unwrap();
    todo!()

}

fn handle_list(
    server: &SharedServerState,
    user: &SharedUser,
    channels: Option<Vec<String>>,
    target: Option<String>,
) -> Result<(), IrcError> {
    todo!()
}


// 