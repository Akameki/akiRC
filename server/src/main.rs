pub mod channel;
pub mod server_state;
pub mod user;

use std::{
    io::{self, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use dns_lookup::lookup_addr;
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
        match stream {
            Ok(st) => {
                let server_clone = server.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(server_clone, st) {
                        eprintln!("{e}");
                    };
                });
            }
            Err(e) => eprintln!("error accepting incoming connection: {e}"),
        }
    }
}

fn handle_connection(server: SharedServerState, stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("{}{}", "Connected: ".green(), addr);

    let mut buf_reader = BufReader::new(stream.try_clone()?);
    let mut buffer = String::new();

    let user = Arc::new(Mutex::new(User::new(stream)));
    let mut registered = false;

    while !registered {
        match blocking_read_message(&mut buf_reader, &mut buffer) {
            Ok(msg) => match msg.command {
                Command::Nick(..) | Command::User(..) => handle_message(&user, msg),
                _ => println!("message from unregistered client {}", msg),
            },
            Err(IrcError::IrcParseError(i, e)) => {
                println!("{} {}", e.bright_purple(), i.bright_purple())
            }
            Err(IrcError::Eof) => {
                println!("{} {}", "Unregistered client disconnected:".red(), addr);
                return Ok(());
            }
            Err(IrcError::Io(e)) => return Err(e),
        }
        registered = try_register_connection(&server, &user)?;
    }

    {
        server.lock().unwrap().print_users();
    }

    loop {
        match blocking_read_message(&mut buf_reader, &mut buffer) {
            Ok(msg) => handle_message(&user, msg),
            Err(IrcError::IrcParseError(i, e)) => {
                println!("{} {}", e.bright_purple(), i.bright_purple())
            }
            Err(IrcError::Eof) => {
                server.lock().unwrap().remove_user(&user);
                println!("{} {}", "Client disconnected:".red(), addr);
                return Ok(());
            }
            Err(IrcError::Io(e)) => return Err(e),
        }
    }
}

fn handle_message(user: &SharedUser, message: Message) {
    use Command::*;

    println!("rec < {message}");
    match message.command {
        Invalid => println!("???"),
        Numeric(_, _) => println!("ignoring numeric {message}"),
        Nick(nick) => user.lock().unwrap().nickname = nick,
        User(username, _, _, _) => user.lock().unwrap().username = username,
    }
}

fn try_register_connection(
    server: &SharedServerState,
    shared_user: &SharedUser,
) -> io::Result<bool> {
    let mut user = shared_user.lock().unwrap();
    if user.nickname.is_empty() || user.username.is_empty() {
        return Ok(false);
    }
    let ip = &user.stream.peer_addr()?.ip();
    user.hostname = lookup_addr(ip).unwrap_or(ip.to_string());
    {
        let server = server.lock().unwrap();
        let mut users = server.users.lock().unwrap();
        if users.contains_key(&user.nickname) {
            // TODO: handle nick in use
            println!(
                "user {} tried joining with taken nick {}",
                user.username, user.nickname
            );
            return Ok(false);
        }
        users.insert(user.nickname.clone(), shared_user.clone());
    }
    let replies = [
        (
            RPL_WELCOME,
            format!(
                ":Welcome to the Internet Relay Network {}!{}@{}",
                user.nickname, user.username, user.hostname
            ),
        ),
        (
            RPL_YOURHOST,
            format!(
                ":Your host is {}, running version {}",
                "akiRC.fake.servername", "ver0"
            ),
        ),
        (RPL_CREATED, format!(":This server was created {}", "?")),
        (
            RPL_MYINFO,
            format!(":{} {} {} {}", "akiRC.fake.servername", "ver0", "", ""),
        ),
    ];
    user.reply(&replies)?;

    Ok(true)
}
