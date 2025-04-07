use std::sync::Arc;

use common::message::{Command, Message, Numeric::*};

use crate::{
    channel::ChannelModes, server_state::SharedServerState, user::{SharedUser, User}, CHANNELMODES, MOTD, NICKLEN, SERVERNAME, TOPICLEN, USERMODES
};

/// Handles one message for a registered user.
pub async fn handle_message(server: &SharedServerState, user: &SharedUser, message: Message) {
    use Command::*;

    match message.command {
        /* Connection Messages */
        // CAP
        // AUTHENTICATE
        // PASS
        NICK { nickname } => handle_NICK(server, user, nickname).await,
        USER { .. } => {
            user.reply(ERR_ALREADYREGISTERED, ":Unauthorized command (already registered)").await
        }
        PING { token } => handle_PING(server, user, token).await,
        PONG { server: _, token: _ } => (),
        // OPER
        QUIT { reason } => handle_QUIT(server, user, reason).await,
        // QUIT { reason } => handle_QUIT(server, user, reason).await,
        ERROR { reason: _ } => (),

        /* Channel Operations */
        JOIN { channels, keys, alt } => handle_JOIN(server, user, channels, keys, alt).await,
        PART { channels, reason } => handle_PART(server, user, channels, reason).await,
        TOPIC { channel, topic } => handle_TOPIC(server, user, channel, topic).await,
        // NAMES
        LIST { channels, elistconds } => handle_LIST(server, user, channels, elistconds).await,
        // INVITE
        // KICK

        /* Server Queries and Commands */
        MOTD { target } => handle_MOTD(server, user, target).await,
        // VERSION
        // ADMIN
        // CONNECT
        // LUSERS
        // TIME
        // STATS
        // HELP
        // INFO
        MODE { target, modestring, modeargs } => {
            handle_MODE(server, user, target, modestring, modeargs).await
        }

        /* Sending Messages */
        PRIVMSG { targets, text } => handle_PRIVMSG(server, user, targets, text).await,
        // NOTICE

        /* User Based Queries */
        WHO { mask } => handle_WHO(server, user, mask).await,
        // WHOIS
        // WHOWAS

        /* Operator Messages */
        // KILL
        // REHASH
        // RESTART
        // SQUIT

        /* Optional Messages */
        // AWAY
        // LINKS
        // USERHOST
        // WALLOPS

        /* Other */
        Invalid(_, Some(num), s) => user.reply(num, &s).await,
        Invalid(_, None, _) | Numeric(..) | Raw(..) => println!("ignoring unexpected message"),
    };
    // println!("{}", server.lock().await);
}

// Individual command handlers follow below.
// Ensure server lock is acquired before user lock.
// Parameters prefixed with a_ are args for the command

// Type aliasing for conciseness and flexibility for future changes
type Sss = SharedServerState;
type Su = SharedUser;
type Res = ();

/* Connection Messages */
// CAP
// AUTHENTICATE
// PASS
#[allow(non_snake_case)]
async fn handle_NICK(sss: &Sss, su: &Su, a_nick: String) -> Res {
    let mut server = sss.lock().await;
    let a_nick = a_nick.chars().take(NICKLEN).collect::<String>();
    let target = su.get_fqn_string();
    if server.try_update_nick(su, &a_nick) {
        su.broadcast(
            true,
            Arc::new(Message::new(Some(&target), Command::NICK { nickname: a_nick })),
        )
        .await
    } else {
        su.reply(ERR_NICKNAMEINUSE, &format!("{} :Nickname is already in use", a_nick)).await
    }
}
#[allow(non_snake_case)]
async fn handle_PING(_sss: &Sss, su: &Su, a_token: String) -> Res {
    su.send(Arc::new(Message::new(
        Some(SERVERNAME),
        Command::PONG { server: SERVERNAME.to_string(), token: a_token },
    )))
    .await;
}
// PONG (ignored)
// OPER
#[allow(non_snake_case)]
async fn handle_QUIT(_sss: &Sss, su: &Su, a_reason: String) -> Res {
    su.broadcast(true, Arc::new(Message::new(
        Some(&su.get_fqn_string()),
        Command::QUIT { reason: a_reason },
    ))).await;
    su.send(Arc::new(Message::new(
        Some(SERVERNAME),
        Command::ERROR { reason: format!(":Closing Link: {} (Client Quit)", su.get_fqn_string()) },
    ))).await;
}
// ERROR (ignored)

/* Channel Operations */
#[allow(non_snake_case)]
async fn handle_JOIN(
    sss: &Sss,
    user: &Su,
    a_channels: Vec<String>,
    _a_keys: Vec<String>, // todo
    _a_flag: bool,        // todo
) -> Res {
    let mut server = sss.lock().await;
    // todo: join multiple channels
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
    if let Some((topic, who, time)) = channel.get_topic_info() {
        user.reply(RPL_TOPIC, &format!("{} :{}", channel_name, topic)).await;
        user.reply(RPL_TOPICWHOTIME, &format!("{} {} {}", channel_name, who, time)).await;
    } else {
        user.reply(RPL_NOTOPIC, &format!("{} :No topic is set", channel_name)).await;
    }
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
async fn handle_TOPIC(sss: &Sss, user: &Su, a_channel: String, a_topic: Option<String>) -> Res {
    let server = sss.lock().await;
    let a_topic = a_topic.map(|s| s.chars().take(TOPICLEN).collect::<String>());
    if let Some(channel) = server.get_channel(&a_channel) {
        if channel.contains_user(user) {
            if let Some(topic) = a_topic {
                channel.set_topic(user, &topic);
                channel
                    .broadcast(Arc::new(Message::new(
                        Some(&user.get_fqn_string()),
                        Command::TOPIC { channel: a_channel.clone(), topic: Some(topic) },
                    )))
                    .await;
            } else if let Some((topic, who, time)) = channel.get_topic_info() {
                user.reply(RPL_TOPIC, &format!("{} :{}", a_channel, topic)).await;
                user.reply(RPL_TOPICWHOTIME, &format!("{} {} {}", a_channel, who, time)).await;
            } else {
                user.reply(RPL_NOTOPIC, &format!("{} :No topic is set", a_channel)).await;
            }
        } else {
            user.reply(ERR_NOTONCHANNEL, ":You're not on that channel").await;
        }
    } else {
        user.reply(ERR_NOSUCHCHANNEL, ":No such channel").await;
    }
}
// NAMES
#[allow(non_snake_case)]
async fn handle_LIST(
    sss: &Sss,
    user: &Su,
    a_channels: Vec<String>,
    _a_elistconds: Option<String>, // todo
) -> Res {
    let server = sss.lock().await;
    user.reply(RPL_LISTSTART, "Channel :Users  Name").await;
    for ch in server.get_channels() {
        if ch.get_modes().s || !a_channels.is_empty() && !a_channels.contains(&ch.name) {
            continue;
        }
        let topic = ch.get_topic_info().map(|(t, _, _)| t).unwrap_or_default();
        user.reply(RPL_LIST, &format!("{} {} :{}", ch.name, ch.user_count(), topic)).await;
    }
    user.reply(RPL_LISTEND, ":End of /LIST").await;
}
// INVITE
// KICK

/* Server Queries and Commands */
#[allow(non_snake_case)]
async fn handle_MOTD(_sss: &Sss, user: &Su, a_target: String) -> Res {
    #[allow(clippy::const_is_empty)]
    if MOTD.is_empty() {
        user.reply(ERR_NOMOTD, ":MOTD File is missng").await;
    } else if a_target.is_empty() || a_target == SERVERNAME {
        user.reply(RPL_MOTDSTART, &format!(":- {} Message of the day -", SERVERNAME)).await;
        for line in MOTD.lines() {
            user.reply(RPL_MOTD, &format!(":- {line}")).await;
        }
        user.reply(RPL_ENDOFMOTD, ":End of /MOTD command").await;
    } else {
        user.reply(ERR_NOSUCHSERVER, ":No such server").await;
    }
}
// VERSION
// ADMIN
// CONNECT
// LUSERS
// TIME
// STATS
// HELP
// INFO
#[allow(non_snake_case)]
async fn handle_MODE(
    sss: &Sss,
    user: &Su,
    a_target: String,
    a_modestring: String,
    _a_modeargs: Vec<String>,
) -> Res {
    let server = sss.lock().await;
    if let Some(target_user) = server.get_user(&a_target) {
        if !User::are_same(user, &target_user) {
            user.reply(ERR_USERSDONTMATCH, ":Cannot change/view modes of other users").await
        } else if a_modestring.is_empty() {
            // get user modes
            let modes: String = user.get_modes().collect();
            user.reply(RPL_UMODEIS, &format!("{} {}", a_target, modes)).await
        } else {
            // set user modes
            let mut mode_iter = a_modestring.chars();
            let mut rep_modestring = String::from("");
            let mut plus_or_minus = mode_iter.next().unwrap();
            let mut unknown = false;
            // TODO: coalesce duplicate modes
            for modechar in mode_iter {
                if modechar == '+' || modechar == '-' {
                    plus_or_minus = modechar;
                } else if !USERMODES.contains(modechar) {
                    unknown = true;
                } else if match plus_or_minus {
                    '+' => user.add_mode(modechar),
                    _ => user.remove_mode(modechar),
                } {
                    rep_modestring.push(plus_or_minus);
                    rep_modestring.push(modechar);
                }
            }
            if unknown {
                user.reply(ERR_UMODEUNKNOWNFLAG, ":Unknown MODE flag").await;
            }
            user.send(Arc::new(Message::new(
                Some(&user.get_fqn_string()),
                Command::MODE {
                    target: a_target.clone(),
                    modestring: rep_modestring,
                    modeargs: vec![],
                },
            )))
            .await
        }
    } else if let Some(channel) = server.get_channel(&a_target) {
        // channel target
        if channel.contains_user(user) {
            if a_modestring.is_empty() {
                // get channel modes
                let ChannelModes { s } = channel.get_modes();
                let mut rep_modestring = String::from("+");
                let rep_modeargs = String::from("");
                // type D: flags
                [(s, 's')].iter().filter(|(b, _)| *b).for_each(|(_, c)| {
                    rep_modestring.push(*c);
                });
                user.reply(
                    RPL_CHANNELMODEIS,
                    &format!("{} {}{}", a_target, rep_modestring, rep_modeargs),
                )
                .await;
                user.reply(RPL_CREATIONTIME, &format!("{} {}", a_target, channel.creation_time))
                    .await
            } else if let Some(invalid) =
                a_modestring.chars().find(|&c| !matches!(c, '+' | '-') && !CHANNELMODES.contains(c))
            {
                // invalid set channel modes
                user.reply(ERR_UNKNOWNMODE, &format!("{invalid} :is unknown mode char to me")).await
            } else {
                // set channel modes
                let mut mode_iter = a_modestring.chars();
                let mut rep_modestring = String::from("");
                let rep_modeargs = Vec::new();
                let mut plus_or_minus = mode_iter.next().unwrap();
                // TODO: coalesce dupes
                for modechar in mode_iter {
                    match modechar {
                        '+' | '-' => plus_or_minus = modechar,
                        // type D: flags
                        's' => {
                            if channel.set_mode_type_d(modechar, plus_or_minus == '+') {
                                rep_modestring.push(plus_or_minus);
                                rep_modestring.push(modechar);
                            }
                        }
                        _ => unreachable!(),
                    }
                }

                channel
                    .broadcast(Arc::new(Message::new(
                        Some(&user.get_fqn_string()),
                        Command::MODE {
                            target: a_target.clone(),
                            modestring: rep_modestring,
                            modeargs: rep_modeargs,
                        },
                    )))
                    .await
            }
        } else {
            user.reply(
                ERR_CHANOPRIVSNEEDED,
                &format!("{} :You're not a channel operator", a_target),
            )
            .await
        }
    }
}

/* Sending Messages */
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
// NOTICE

/* User Based Queries */
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
                        "{} {} {} {} {} H :0 {}",
                        mask,
                        u.username,
                        u.hostname,
                        SERVERNAME,
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
                    "* {} {} {} {} H :0 {}",
                    u.username,
                    u.hostname,
                    SERVERNAME,
                    u.get_nickname(),
                    u.realname
                )
            };
            su.reply(RPL_WHOREPLY, &reply).await;
        }
    }
    su.reply(RPL_ENDOFWHO, &format!("{} :End of WHO list", mask)).await;
}
// WHOIS
// WHOWAS

/* Operator Messages */
// KILL
// REHASH
// RESTART
// SQUIT

/* Optional Messages */
// AWAY
// LINKS
// USERHOST
// WALLOPS
