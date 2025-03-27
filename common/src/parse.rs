use crate::*;
use std::str::FromStr;

impl FromStr for Command {
    type Err = IrcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!("Command::from_str not implemented: {s}")
    }
}

impl FromStr for Message {
    type Err = IrcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        let cmd = words.next().unwrap();
        let params = words.map(|word| word.to_string()).collect();

        let command = Command::new(cmd, params); // todo: use Command::from_str to parse args

        match command {
            Command::Invalid() => Err(IrcError::IrcParseError(format!(
                "Error parsing a Message from: {s}"
            ))),
            _ => Ok(Message {
                prefix: None,
                command,
            }),
        }
    }
}
