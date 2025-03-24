use core::str;
use std::{
    io::{self, BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

use common::*;

fn main() {
    println!("i am server!");
    let listener = TcpListener::bind("127.0.0.1:9999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(st) => {
                handle_connection(st).expect("error handling connection");
            }
            Err(e) => panic!("uhoh: {e}"),
        }
    }
}

struct User {
    user: Option<String>,
    nick: Option<String>,
    registered: bool,
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("Connected: {}", addr);

    let mut buf_reader = BufReader::new(stream.try_clone()?);

    let mut user = User {
        user: None,
        nick: None,
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

        if let (false, Some(nick), Some(username)) = (&mut user.registered, &user.nick, &user.user)
        {
            let msg = Message::new(
                Some(String::from("akiRC")),
                Command::Numeric(
                    Numeric::RPL_WELCOME,
                    vec![
                        username.clone(),
                        format!(
                            ":Welcome to the Internet Relay Network {}!{}@{}",
                            nick.clone(),
                            username.clone(),
                            addr.ip().to_string()
                        ),
                    ],
                ),
            );
            // send message
            let msg_str = format!("{}\r\n", msg);
            println!("Sending: {msg_str}");
            stream.write_all(msg_str.as_bytes())?;

            user.registered = true;
        }
    }
    println!("Disconnected: {addr}");
    Ok(())
}

fn handle_message(s: &str, user: &mut User) {
    use common::Command::*;
    let Ok(msg): Result<Message, String> = s.parse() else {
        return;
    };

    println!("{msg:#?}");
    match msg.command {
        Invalid() => println!("[unrecognized] {s}"),
        Numeric(_, _) => println!("!? client sent numeric: {s}"),
        Nick(nickname) => user.nick = Some(nickname),
        User(username, _, _, _) => user.user = Some(username),
    }
}