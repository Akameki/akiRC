use owo_colors::{self, OwoColorize};
use std::{
    io::{BufRead, BufReader, Write, stdin},
    net::TcpStream,
};

// CLI program that sends and receives text to a TcpStream.
fn main() {
    let addr = "irc.libera.chat:6667";
    let mut stream = TcpStream::connect(addr).unwrap();
    println!("=====\n{}", "Connected".green().italic());
    let stream_clone = stream.try_clone().unwrap();
    std::thread::spawn(move || {
        let mut buf_reader = BufReader::new(stream_clone);
        let mut buffer = String::new();
        loop {
            match buf_reader.read_line(&mut buffer).unwrap() {
                0 => break, // EOF
                _ => println!("{}", buffer.bright_blue()),
            }
            buffer.clear();
        }
        println!("\n{}=====", "Disconnected".red().italic());
        std::process::exit(1);
    });

    let mut input = String::new();
    loop {
        match stdin().read_line(&mut input).unwrap() {
            0 => break, // EOF
            _ => stream.write_all(input.as_bytes()).unwrap(),
        }
        input.clear();
    }
}
