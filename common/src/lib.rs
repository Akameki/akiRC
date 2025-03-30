pub mod parse;
pub mod stream_handler;
pub mod message;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IrcError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("EOF")]
    Eof,
    #[error("Error {1} parsing {0}")]
    IrcParseError(String, String),
}
