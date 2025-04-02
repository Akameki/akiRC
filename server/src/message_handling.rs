use std::io;

use common::message::{Command, Message, Numeric::*};

use crate::{server_state::SharedServerState, user::SharedUser};

/// Message handling AFTER connection registration/handshake
pub fn handle_message(
    server: &SharedServerState,
    user: &SharedUser,
    message: Message,
) -> io::Result<()> {
    use Command::*;

    match message.command {
        Nick(nick) => handle_nick(server, user, nick),
        User(..) => Ok(user.lock().unwrap().reply(
            ERR_ALREADYREGISTERED,
            ":Unauthorized command (already registered)",
        )?),
        Join(channels, keys, flag) => handle_join(server, user, channels, keys, flag),
        List(channels, target) => handle_list(server, user, channels, target),
        Invalid | Numeric(..) => {
            println!("ignoring unexpected message");
            Ok(())
        }
    }?;
    println!("handled!");
    Ok(())
}

// Individual command handlers follow below.
// Ensure server lock is acquired before user lock.
// Parameters prefixed with a_ are args for the command

// Type aliasing for conciseness and flexibility for future changes
type Sss = SharedServerState;
type Su = SharedUser;
type Res = io::Result<()>;

fn handle_nick(sss: &Sss, su: &Su, a_nick: String) -> Res {
    let mut server = sss.lock().unwrap();
    if server.try_update_nick(su, &a_nick) {
        let target = su.lock().unwrap().target_str();
        server.broadcast(&[Message::new(Some(&target), Command::Nick(a_nick))])
    } else {
        su.lock().unwrap().reply(
            ERR_NICKNAMEINUSE,
            &format!("{} :Nickname is already in use", a_nick),
        )
    }
}

fn handle_join(
    sss: &Sss,
    su: &Su,
    a_channels: Vec<String>,
    a_keys: Vec<String>,
    a_flag: bool,
) -> Res {
    let mut server = sss.lock().unwrap();
    let user = su.lock().unwrap();
    // TODO: assuming just one channel for now
    let channel_name = a_channels[0].clone();
    let channel = if let Some(ch) = server.get_channel(&channel_name) {
        ch
    } else {
        server.create_channel(&channel_name)
    };
    channel.add_user(su);
    let source = user.target_str();
    drop(user);
    let nicks = channel.get_user_nicks();
    channel.broadcast(&Message::new(
        Some(&source),
        Command::Join(vec![channel_name.clone()], vec![], false),
    ))?;
    let user = su.lock().unwrap();
    user.reply(
        RPL_NAMREPLY,
        &format!("= {} :{}", channel_name, nicks.join(" ")),
    )?;
    user.reply(
        RPL_ENDOFNAMES,
        &format!("{} :End of /NAMES list", channel_name),
    )
}

fn handle_list(
    sss: &Sss,
    su: &Su,
    a_channels: Option<Vec<String>>,
    a_target: Option<String>,
) -> Res {
    let server = sss.lock().unwrap();
    let user = su.lock().unwrap();
    if a_channels.is_none() {
        user.reply(RPL_LISTSTART, "Channel :Users  Name")?;
        for ch in server.get_channels() {
            user.reply(
                RPL_LIST,
                &format!("{} {} :{}", ch.name, ch.user_count(), ch.topic),
            )?;
        }
        user.reply(RPL_LISTEND, ":End of /LIST")?;
    } else {
        todo!()
    }
    Ok(())
}
