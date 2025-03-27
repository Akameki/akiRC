pub mod channel;
pub mod user;

use channel::SharedChannel;
use core::str;
use dns_lookup::lookup_addr;
use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    io::{self, BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use common::*;
use common::{Numeric::*, stream_handler::blocking_read_message};
use user::{SharedUser, User};

pub struct ServerState {
    pub users: Arc<Mutex<HashMap<TcpStream, SharedUser>>>,
    pub channels: Arc<Mutex<HashMap<String, SharedChannel>>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            users: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn main() {
    println!("i am server!");
    let listener = TcpListener::bind("0.0.0.0:9999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(st) => {
                thread::spawn(|| {
                    if let Err(e) = handle_connection(st) {
                        eprintln!("{e}");
                    };
                });
            }
            Err(e) => eprintln!("error accepting incoming connection: {e}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("{}{}", "Connected: ".green(), addr);

    let mut buf_reader = BufReader::new(stream.try_clone()?);
    let mut buffer = String::new();

    let mut user = User {
        username: String::from(""),
        nickname: String::from(""),
        hostname: String::from(""),
        realname: String::from(""),
        registered: false,
    };

    loop {
        let next_msg = blocking_read_message(&mut buf_reader, &mut buffer);
        match next_msg {
            Ok(m) => handle_message(m, &mut user),
            Err(IrcError::IrcParseError(s)) => println!("{}", s.bright_purple()),
            Err(IrcError::Io(e)) => return Err(e),
            Err(IrcError::Eof) => break,
        }
        if !user.registered && !user.nickname.is_empty() && !user.username.is_empty() {
            register_connection(&mut stream, &mut user)?;
        }
    }

    // on EOF
    println!("{}{}", "Disconnected: ".red(), addr);
    Ok(())
}

fn handle_message(message: Message, user: &mut User) {
    use common::Command::*;

    println!("rec < {message}");
    match message.command {
        Invalid() => println!("???"),
        Numeric(_, _) => println!("ignoring numeric {message}"),
        Nick(nick) => user.nickname = nick,
        User(username, _, _, _) => user.username = username,
    }
}

fn register_connection(stream: &mut TcpStream, user: &mut User) -> io::Result<()> {
    user.hostname =
        lookup_addr(&stream.peer_addr()?.ip()).unwrap_or(stream.peer_addr()?.ip().to_string());
    let mut buffer = Vec::new();
    write!(
        buffer,
        "{}\r\n{}\r\n{}\r\n{}\r\n",
        Message::new_numeric(
            "akiRC",
            RPL_WELCOME,
            &user.username,
            &format!(
                ":Welcome to the Internet Relay Network {}!{}@{}",
                user.nickname, user.username, user.hostname
            )
        ),
        Message::new_numeric(
            "akiRC",
            RPL_YOURHOST,
            &user.username,
            &format!(
                ":Your host is {}, running version {}",
                "akiRC.fake.servername", "ver0"
            )
        ),
        Message::new_numeric(
            "akiRC",
            RPL_CREATED,
            &user.username,
            &format!(":This server was created {}", "?")
        ),
        Message::new_numeric(
            "akiRC",
            RPL_MYINFO,
            &user.username,
            &format!(":{} {} {} {}", "akiRC.fake.servername", "ver0", "", "")
        )
    )?;

    println!("send > {}", str::from_utf8(&buffer).expect("utf8 string?"));
    stream.write_all(&buffer)?;

    user.registered = true;

    Ok(())
}
