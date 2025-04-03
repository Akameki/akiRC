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
        User(..) => Ok(user.try_lock().unwrap().reply(
            ERR_ALREADYREGISTERED,
            ":Unauthorized command (already registered)",
        )?),
        Join(channels, keys, flag) => handle_join(server, user, channels, keys, flag),
        List(channels, target) => handle_list(server, user, channels, target),
        WHO { mask } => handle_WHO(server, user, mask),
        PRIVMSG { targets, text } => handle_PRIVMSG(server, user, targets, text),
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
        let target = su.try_lock().unwrap().target_str();
        server.broadcast(&[Message::new(Some(&target), Command::Nick(a_nick))])
    } else {
        su.try_lock().unwrap().reply(
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
    let user = su.try_lock().unwrap();
    // TODO: assuming just one channel for now
    let channel_name = a_channels[0].clone();
    let channel = if let Some(ch) = server.get_channel_mut(&channel_name) {
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
    let user = su.try_lock().unwrap();
    user.reply(
        RPL_NAMREPLY,
        &format!("= {} :{}", channel_name, nicks.join(" ")), // todo: message limit
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
    let user = su.try_lock().unwrap();
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

#[allow(non_snake_case)]
fn handle_WHO(sss: &Sss, su: &Su, mask: String) -> Res {
    let server = sss.lock().unwrap();
    // let user = su.try_lock().unwrap();

    if mask.starts_with("#") {
        // todo: other prefixes
        if let Some(channel) = server.get_channel(&mask) {
            for masked_user in channel.get_users() {
                let reply = {
                    let u = masked_user.try_lock().unwrap();
                    format!(
                        "{} {} {} akiRC {} H :0 {}",
                        mask,
                        u.username,
                        u.hostname,
                        u.get_nickname(),
                        u.realname
                    )
                };
                su.try_lock().unwrap().reply(RPL_WHOREPLY, &reply)?;
            }
        }
    } else {
        // todo: user masks
        if let Some(masked_user) = server.get_user(&mask) {
            let reply = {
                let u = masked_user.try_lock().unwrap();
                format!(
                    "* {} {} akiRC {} H :0 {}",
                    u.username,
                    u.hostname,
                    u.get_nickname(),
                    u.realname
                )
            };
            su.try_lock().unwrap().reply(RPL_WHOREPLY, &reply)?;
        }
    }
    su.try_lock()
        .unwrap()
        .reply(RPL_ENDOFWHO, &format!("{} :End of WHO list", mask))
}

#[allow(non_snake_case)]
fn handle_PRIVMSG(sss: &Sss, su: &Su, targets: Vec<String>, text: String) -> Res {
    let server = sss.lock().unwrap();
    let nick = su.try_lock().unwrap().get_nickname();
    
    for target in targets {
        let success_msg = Message::new(
            Some(&nick),
            Command::PRIVMSG {
                targets: vec![target.clone()],
                text: text.to_owned(),
            },
        );
        if let Some(channel) = server.get_channel(&target) {
            for user in channel.get_users() {
                let user_lock = user.try_lock().unwrap();
                if user_lock.get_nickname() != *nick {
                    user_lock.send(&[&success_msg])?;
                }
            }
        } else if let Some(user) = server.get_user(&target) {
            user.try_lock().unwrap().send(&[&success_msg])?;
        } else {
            su.try_lock().unwrap().reply(ERR_NOSUCHNICK, ":No such nick/channel")?;
        }
    }
    Ok(())
}
