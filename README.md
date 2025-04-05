# akiRC
A simple IRC server built with Rust, designed to handle a large number of connections with Tokio.

This project also includes a crate for representing and parsing IRC messages in the `common` package.

## Example usage
With rust and `cargo` installed, simply clone and build/run on your local machine.

```
git clone https://github.com/yourusername/akiRC.git
cd akiRC
cargo run --release -p server
```
The server will listen to all interfaces on port 9999.

## License
Everything licensed under the [MIT license].

[MIT license]: https://github.com/Akameki/akiRC/blob/main/LICENSE