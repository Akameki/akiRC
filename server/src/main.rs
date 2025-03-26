pub mod channel;
pub mod user;

use channel::SharedChannel;
use core::str;
use dns_lookup::lookup_addr;
use std::{
    collections::HashMap,
    io::{self, BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use common::Numeric::*;
use common::*;
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
            // Ok(st) => handle_connection(st).expect("error handling connection"),
            Ok(st) => {
                thread::spawn(|| handle_connection(st));
            }
            Err(e) => panic!("uhoh: {e}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("Connected: {}", addr);

    let mut buf_reader = BufReader::new(stream.try_clone()?);

    let mut user = User {
        username: String::from(""),
        nickname: String::from(""),
        hostname: String::from(""),
        realname: String::from(""),
        registered: false,
    };

    loop {
        // let mut buf: [u8; 512] = [0; 512];
        let mut str = String::new();
        if buf_reader.read_line(&mut str)? == 0 {
            break;
        }

        // separate messages by \r and \n, ignoring empty messages.
        str.split("\r")
            .flat_map(|m| m.split("\n"))
            .filter(|x| !x.is_empty())
            .for_each(|m| handle_message(m, &mut user));

        // if let (false, Some(nick), Some(username)) = (&mut user.registered, &user.nick, &user.user)

        if !user.registered && !user.nickname.is_empty() && !user.username.is_empty() {
            register_connection(&mut stream, &mut user)?;
        }
    }
    println!("Disconnected: {addr}");
    Ok(())
}

fn handle_message(s: &str, user: &mut User) {
    use common::Command::*;
    let Ok(msg): Result<Message, String> = s.parse() else {
        return println!("rec [unparsed] < {s}");
    };

    println!("rec < {msg}");
    match msg.command {
        Invalid() => println!("[unrecognized] {s}"),
        Numeric(_, _) => println!("!? client sent numeric: {s}"),
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
