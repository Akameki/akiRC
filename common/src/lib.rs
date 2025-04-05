pub mod message;
pub mod parse;
// pub mod stream_handler;

use thiserror::Error;

pub type IrcParseErrorString = String;

#[derive(Error, Debug)]
pub enum IrcError {
    #[error("IrcError::I: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    IrcParseError(IrcParseErrorString),
}

impl From<nom::Err<nom::error::Error<&str>>> for IrcError {
    fn from(value: nom::Err<nom::error::Error<&str>>) -> Self {
        match value {
            nom::Err::Incomplete(e) => {
                Self::IrcParseError(format!("unexpected nom::Err::Incomplete {:?}", e))
            }
            nom::Err::Error(e) => {
                Self::IrcParseError(format!("Error {} parsing {}", e.code.description(), e.input))
            }
            nom::Err::Failure(f) => {
                Self::IrcParseError(format!("Error {} parsing {}", f.code.description(), f.input))
            }
        }
    }
}

impl From<IrcParseErrorString> for IrcError {
    fn from(value: IrcParseErrorString) -> Self {
        Self::IrcParseError(value)
    }
}
