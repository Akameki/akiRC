mod channel;
mod message_handling;
mod server_state;
mod user;

use std::{
    io::{self},
    sync::Arc,
};

use common::{
    IrcError,
    message::{Command, Message, Numeric::*},
};
use dns_lookup::lookup_addr;
use message_handling::handle_message;
use owo_colors::OwoColorize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, tcp::OwnedReadHalf},
    sync::{Mutex, mpsc},
    task,
};

use crate::{
    server_state::{ServerState, SharedServerState},
    user::{SharedUser, User},
};

#[tokio::main]
async fn main() {
    let server = Arc::new(Mutex::new(ServerState::new()));
    let listener = TcpListener::bind("0.0.0.0:9999").await.unwrap();

    println!("{}", "akiRC server started!".underline());
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                continue;
            }
        };

        let server_clone = server.clone();
        task::spawn(async move {
            if let Err(e) = handle_connection(server_clone, stream).await {
                eprintln!("{}", e.red());
            };
        });
    }
}

enum MaybeReg {
    Unreg(User),
    Reg(SharedUser),
}

async fn handle_connection(server: SharedServerState, stream: TcpStream) -> io::Result<()> {
    let addr = stream.peer_addr()?;
    println!("{} {} Looking up hostname...", "Connected:".green(), addr);

    let (reader, mut writer) = stream.into_split();
    let (tx, mut rx) = mpsc::channel::<Arc<Message>>(100);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("{}", format!("-> {msg}").truecolor(100, 110, 135));
            if let Err(e) = writer.write_all((msg.to_string() + "\r\n").as_bytes()).await {
                eprintln!("Write error: {}", e);
                break;
            }
        }
    });

    let mut buf_reader = BufReader::new(reader);
    let mut buffer = String::new();

    let ip = addr.ip();
    // todo: Ident
    println!("(Looking up hostname)");
    let hostname = lookup_addr(&ip).unwrap_or(ip.to_string());

    let mut user = MaybeReg::Unreg(User::new(tx, hostname));

    loop {
        match next_message(&mut buf_reader, &mut buffer).await {
            Ok(msg) => match user {
                MaybeReg::Unreg(u) => user = handle_message_and_try_register(&server, u, msg).await,
                MaybeReg::Reg(ref u) => handle_message(&server, u, msg).await,
            },
            Err(IrcError::IrcParseError(e)) => println!("{}", e.bright_purple()),
            Err(IrcError::Io(e)) => {
                println!("{} {} [{}] {e}", "Disconnected:".red(), addr, e.kind());
                match user {
                    MaybeReg::Unreg(u) => server.lock().await.remove_unregistered_nick(u),
                    MaybeReg::Reg(u) => server.lock().await.remove_user(u),
                }
                return Ok(());
            }
        }
    }
}

async fn handle_message_and_try_register(
    server: &SharedServerState,
    mut user: User,
    message: Message,
) -> MaybeReg {
    let mut server_lock = server.lock().await;

    match message.command {
        Command::NICK { nickname: new_nick } => {
            if !server_lock.try_update_unregistered_nick(&user.get_nickname(), &new_nick) {
                user.reply(ERR_NICKNAMEINUSE, &format!("{} :Nickname is already in use", new_nick))
                    .await;
            } else {
                user.set_nickname(&new_nick);
            }
        }
        Command::USER { username, _1, _2, realname } => {
            user.username = format!("~{username}");
            user.realname = realname;
        }
        Command::Invalid(nick, Some(num), s) if ["NICK", "USER"].contains(&nick.as_str()) => {
            user.reply(num, &s).await;
        }
        _ => println!("Ignoring message from unregistered user: ({})", message),
    }

    if user.username.is_empty() || user.get_nickname().is_empty() {
        return MaybeReg::Unreg(user);
    }

    let user = server_lock.register_user(user);

    user.reply(
        RPL_WELCOME,
        &format!(
            ":Welcome to the Internet Relay Network {}!{}@{}",
            user.get_nickname(),
            user.username,
            user.hostname
        ),
    )
    .await;
    user.reply(
        RPL_YOURHOST,
        &format!(":Your host is {}, running version {}", "akiRC.fake.servername", "ver0"),
    )
    .await;
    user.reply(RPL_CREATED, &format!(":This server was created {}", "?")).await;
    user.reply(RPL_MYINFO, &format!("{} {} {} {}", "akiRC.fake.servername", "ver0", "", "")).await;
    MaybeReg::Reg(user)
}

async fn next_message(
    reader: &mut BufReader<OwnedReadHalf>,
    // reader: &mut (impl AsyncBufReadExt + Unpin),
    buffer: &mut String,
) -> Result<Message, IrcError> {
    loop {
        if let Some(pos_rn) = buffer.find(['\r', '\n']) {
            let mut msg_str: String = buffer.drain(..(pos_rn + 1)).collect();
            msg_str.pop();
            if !msg_str.is_empty() {
                let res: Result<Message, IrcError> = msg_str.parse();
                match &res {
                    Ok(m) if matches!(m, Message { command: Command::Invalid(..), .. }) => {
                        println!("{}", format!("<- [{msg_str}] -- {m}").bright_purple())
                    }
                    Ok(_) => {
                        println!("{}", format!("<- {msg_str}").bright_blue())
                    }
                    Err(err) => println!("{}", format!("<- [{msg_str}] -- {err}").bright_red()),
                }
                return res;
            }
        } else if reader.read_line(buffer).await? == 0 {
            return Err(IrcError::Io(io::Error::from(io::ErrorKind::UnexpectedEof)));
        }
    }
}
