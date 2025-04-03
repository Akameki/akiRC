mod channel;
mod message_handling;
mod server_state;
mod user;

use std::{
    io::{self, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use dns_lookup::lookup_addr;
use message_handling::handle_message;
use owo_colors::OwoColorize;

use crate::{
    server_state::{ServerState, SharedServerState},
    user::{SharedUser, User},
};

use common::{
    IrcError,
    message::{Command, Message, Numeric::*},
    stream_handler::blocking_read_message,
};

fn main() {
    let server = Arc::new(Mutex::new(ServerState::new()));
    let listener = TcpListener::bind("0.0.0.0:9999").unwrap();

    println!("{}", "akiRC server started!".underline());
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            eprintln!(
                "error accepting incoming connection: {}",
                stream.unwrap_err()
            );
            continue;
        };
        let server_clone = server.clone();
        thread::spawn(move || {
            if let Err(e) = handle_connection(server_clone, stream) {
                eprintln!("{e}");
            };
        });
    }
}

fn handle_connection(server: SharedServerState, stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("{}{}", "Connected: ".green(), addr);
    let ip = addr.ip();

    let mut buf_reader = BufReader::new(stream.try_clone()?);
    let mut buffer = String::new();

    let user = Arc::new(Mutex::new(User::new(stream)));
    let mut registered = false;

    println!("(Looking up hostname)");
    user.try_lock().unwrap().hostname = lookup_addr(&ip).unwrap_or(ip.to_string());

    loop {
        match blocking_read_message(&mut buf_reader, &mut buffer) {
            Ok(msg) if registered => handle_message(&server, &user, msg)?,
            Ok(msg) => registered = handle_message_and_try_register(&server, &user, msg)?,
            Err(IrcError::IrcParseError(e)) => println!("{}", e.bright_purple()),
            Err(IrcError::Eof) => {
                server.lock().unwrap().remove_user(&user);
                println!("{} {}", "Disconnected:".red(), addr);
                return Ok(());
            }
            Err(IrcError::Io(e)) => return Err(e),
        }
    }
}

fn handle_message_and_try_register(
    server: &SharedServerState,
    user: &SharedUser,
    message: Message,
) -> io::Result<bool> {
    let mut server_lock = server.lock().unwrap();
    let mut user_lock = user.try_lock().unwrap();

    match message.command {
        Command::Nick(nick) => {
            if server_lock.contains_nick(&nick) {
                if user_lock.get_nickname() != nick {
                    user_lock.reply(
                        ERR_NICKNAMEINUSE,
                        &format!("{} :Nickname is already in use", nick),
                    )?;
                }
            } else if user_lock.get_nickname().is_empty() {
                user_lock.set_nickname(&nick);
                server_lock.insert_user(&nick, user);
            } else {
                assert!(server_lock.try_update_nick(user, &nick));
            }
        }
        Command::User(username, mode, _, realname) => {
            user_lock.username = format!("~{username}");
            user_lock.realname = realname;
        }
        _ => println!("Ignoring message from unregistered user: ({})", message),
    }

    if user_lock.username.is_empty() || user_lock.get_nickname().is_empty() {
        return Ok(false);
    }

    // todo: after implementing tokio, wait for ident/hostname
    user_lock.registered = true;

    user_lock.reply_multiple(replies![
        RPL_WELCOME => ":Welcome to the Internet Relay Network {}!{}@{}",
            user_lock.get_nickname(), user_lock.username, user_lock.hostname;
        RPL_YOURHOST => ":Your host is {}, running version {}",
            "akiRC.fake.servername", "ver0";
        RPL_CREATED => ":This server was created {}",
            "?";
        RPL_MYINFO => "{} {} {} {}",
            "akiRC.fake.servername", "ver0", "", "";
    ])?;
    Ok(true)
}
