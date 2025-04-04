use std::sync::Arc;

use common::message::{Command, Message, Numeric::*};

use crate::{server_state::SharedServerState, user::SharedUser};

/// Handles one message for a registered user.
pub async fn handle_message(server: &SharedServerState, user: &SharedUser, message: Message) {
    use Command::*;

    match message.command {
        NICK { nickname } => handle_nick(server, user, nickname).await,
        USER { .. } => {
            user.reply(ERR_ALREADYREGISTERED, ":Unauthorized command (already registered)").await
        }
        JOIN { channels, keys, alt } => handle_join(server, user, channels, keys, alt).await,
        LIST { channels, elistconds } => handle_list(server, user, channels, elistconds).await,
        WHO { mask } => handle_WHO(server, user, mask).await,
        PRIVMSG { targets, text } => handle_PRIVMSG(server, user, targets, text).await,
        Invalid | Numeric(..) => println!("ignoring unexpected message"),
    };
    println!("handled!");
}

// Individual command handlers follow below.
// Ensure server lock is acquired before user lock.
// Parameters prefixed with a_ are args for the command

// Type aliasing for conciseness and flexibility for future changes
type Sss = SharedServerState;
type Su = SharedUser;
type Res = ();

async fn handle_nick(sss: &Sss, su: &Su, a_nick: String) -> Res {
    let mut server = sss.lock().await;
    if server.try_update_nick(su, &a_nick) {
        let target = su.fqn_string();
        server
            .broadcast(Arc::new(Message::new(Some(&target), Command::NICK { nickname: a_nick })))
            .await
    } else {
        su.reply(ERR_NICKNAMEINUSE, &format!("{} :Nickname is already in use", a_nick)).await
    }
}

async fn handle_join(
    sss: &Sss,
    su: &Su,
    a_channels: Vec<String>,
    a_keys: Vec<String>,
    a_flag: bool,
) -> Res {
    let mut server = sss.lock().await;
    let user = su;
    // TODO: assuming just one channel for now
    let channel_name = a_channels[0].clone();
    let channel = if let Some(ch) = server.get_channel_mut(&channel_name) {
        ch
    } else {
        server.create_channel(&channel_name)
    };
    channel.add_user(su);
    let source = user.fqn_string();
    let nicks = channel.get_user_nicks();
    channel
        .broadcast(Arc::new(Message::new(
            Some(&source),
            Command::JOIN { channels: vec![channel_name.clone()], keys: vec![], alt: false },
        )))
        .await;
    let user = su;
    user.reply(
        RPL_NAMREPLY,
        &format!("= {} :{}", channel_name, nicks.join(" ")), // todo: message limit
    )
    .await;
    user.reply(RPL_ENDOFNAMES, &format!("{} :End of /NAMES list", channel_name)).await;
}

async fn handle_list(sss: &Sss, su: &Su, a_channels: Vec<String>, a_target: Option<String>) -> Res {
    let server = sss.lock().await;
    let user = su;
    if a_channels.is_empty() {
        user.reply(RPL_LISTSTART, "Channel :Users  Name").await;
        for ch in server.get_channels() {
            user.reply(RPL_LIST, &format!("{} {} :{}", ch.name, ch.user_count(), ch.topic)).await;
        }
        user.reply(RPL_LISTEND, ":End of /LIST").await;
    } else {
        todo!()
    }
}

#[allow(non_snake_case)]
async fn handle_WHO(sss: &Sss, su: &Su, mask: String) -> Res {
    let server = sss.lock().await;

    if mask.starts_with("#") {
        // todo: other prefixes
        if let Some(channel) = server.get_channel(&mask) {
            for masked_user in channel.get_users() {
                let reply = {
                    let u = masked_user;
                    format!(
                        "{} {} {} akiRC {} H :0 {}",
                        mask,
                        u.username,
                        u.hostname,
                        u.get_nickname(),
                        u.realname
                    )
                };
                su.reply(RPL_WHOREPLY, &reply).await;
            }
        }
    } else {
        // todo: user masks
        if let Some(masked_user) = server.get_user(&mask) {
            let reply = {
                let u = masked_user;
                format!(
                    "* {} {} akiRC {} H :0 {}",
                    u.username,
                    u.hostname,
                    u.get_nickname(),
                    u.realname
                )
            };
            su.reply(RPL_WHOREPLY, &reply).await;
        }
    }
    su.reply(RPL_ENDOFWHO, &format!("{} :End of WHO list", mask)).await;
}

#[allow(non_snake_case)]
async fn handle_PRIVMSG(sss: &Sss, su: &Su, targets: Vec<String>, text: String) -> Res {
    let server = sss.lock().await;
    let nick = su.get_nickname();

    for target in targets {
        let success_msg = Arc::new(Message::new(
            Some(&nick),
            Command::PRIVMSG { targets: vec![target.clone()], text: text.to_owned() },
        ));
        if let Some(channel) = server.get_channel(&target) {
            for user in channel.get_users() {
                let user_lock = user;
                if user_lock.get_nickname() != *nick {
                    user_lock.send(Arc::clone(&success_msg)).await;
                }
            }
        } else if let Some(user) = server.get_user(&target) {
            user.send(Arc::clone(&success_msg)).await;
        } else {
            su.reply(ERR_NOSUCHNICK, ":No such nick/channel").await;
        }
    }
}
