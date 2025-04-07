# akiRC
A simple IRC server built with Rust.

Currently supports: NICK, USER, PING, QUIT, JOIN, PART, TOPIC, LIST, MOTD, MODE, PRIVMSG, WHO.


## Example usage
With rust and `cargo` installed, simply clone and build/run on your local machine.

```
git clone https://github.com/yourusername/akiRC.git
cd akiRC
cargo run --release -p server
```
The server will listen to all interfaces on port 6667.

The cargo workspace also includes a library for representing and parsing IRC messages in the `common` package.  
There is also a tiny `client` binary that sends and receives lines over a TcpStream that can be used to connect to an IRC server.
## License
akiRC is licensed under the [MIT license].

[MIT license]: https://github.com/Akameki/akiRC/blob/main/LICENSE