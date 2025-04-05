use std::sync::Arc;

use common::message::{Command, Message, Numeric::*};

use crate::{server_state::SharedServerState, user::SharedUser};

/// Handles one message for a registered user.
pub async fn handle_message(server: &SharedServerState, user: &SharedUser, message: Message) {
    use Command::*;

    match message.command {
        NICK { nickname } => handle_NICK(server, user, nickname).await,
        USER { .. } => {
            user.reply(ERR_ALREADYREGISTERED, ":Unauthorized command (already registered)").await
        }
        /* Channel Operations */
        JOIN { channels, keys, alt } => handle_JOIN(server, user, channels, keys, alt).await,
        PART { channels, reason } => handle_PART(server, user, channels, reason).await,
        LIST { channels, elistconds } => handle_LIST(server, user, channels, elistconds).await,
        WHO { mask } => handle_WHO(server, user, mask).await,
        PRIVMSG { targets, text } => handle_PRIVMSG(server, user, targets, text).await,
        Invalid(_, Some(num), s) => user.reply(num, &s).await,
        Invalid(_, None, _) | Numeric(..) | Raw(..) => println!("ignoring unexpected message"),
    };
    println!("{}", server.lock().await);
}

// Individual command handlers follow below.
// Ensure server lock is acquired before user lock.
// Parameters prefixed with a_ are args for the command

// Type aliasing for conciseness and flexibility for future changes
type Sss = SharedServerState;
type Su = SharedUser;
type Res = ();

#[allow(non_snake_case)]
async fn handle_NICK(sss: &Sss, su: &Su, a_nick: String) -> Res {
    let mut server = sss.lock().await;
    if server.try_update_nick(su, &a_nick) {
        let target = su.get_fqn_string();
        server
            .broadcast(Arc::new(Message::new(Some(&target), Command::NICK { nickname: a_nick })))
            .await
    } else {
        su.reply(ERR_NICKNAMEINUSE, &format!("{} :Nickname is already in use", a_nick)).await
    }
}

#[allow(non_snake_case)]
async fn handle_JOIN(
    sss: &Sss,
    user: &Su,
    a_channels: Vec<String>,
    a_keys: Vec<String>,
    a_flag: bool,
) -> Res {
    let mut server = sss.lock().await;
    // TODO: assuming just one channel for now
    let channel_name = a_channels[0].clone();
    let channel =
        server.get_channel(&channel_name).unwrap_or_else(|| server.create_channel(&channel_name));
    server.add_user_to_channel(user, &channel);
    let nicks = channel.get_users().map(|u| u.get_nickname()).collect::<Vec<_>>();
    channel
        .broadcast(Arc::new(Message::new(
            Some(&user.get_fqn_string()),
            Command::JOIN { channels: vec![channel_name.clone()], keys: vec![], alt: false },
        )))
        .await;
    user.reply(
        RPL_NAMREPLY,
        &format!("= {} :{}", channel_name, nicks.join(" ")), // todo: message limit
    )
    .await;
    user.reply(RPL_ENDOFNAMES, &format!("{} :End of /NAMES list", channel_name)).await;
}

#[allow(non_snake_case)]
async fn handle_PART(sss: &Sss, user: &Su, a_channels: Vec<String>, a_reason: String) -> Res {
    let mut server = sss.lock().await;
    for channel_name in a_channels {
        let success_msg = Arc::new(Message::new(
            Some(&user.get_fqn_string()),
            Command::PART { channels: vec![channel_name.clone()], reason: a_reason.clone() },
        ));
        if let Some(channel) = server.get_channel(&channel_name) {
            if channel.contains_user(user) {
                channel.broadcast(success_msg).await;
                server.remove_user_from_channel(user, &channel);
            } else {
                user.reply(ERR_NOTONCHANNEL, ":You're not on that channel").await;
            }
        } else {
            user.reply(ERR_NOSUCHCHANNEL, ":No such channel").await;
        }
    }
}

#[allow(non_snake_case)]
async fn handle_LIST(
    sss: &Sss,
    user: &Su,
    a_channels: Vec<String>,
    a_elistconds: Option<String>,
) -> Res {
    let server = sss.lock().await;
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
