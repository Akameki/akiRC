use crate::*;
use std::str::FromStr;

impl FromStr for Command {
    type Err = String; // todo

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!("Command from_str not implemented: {s}")
    }
}

impl FromStr for Message {
    type Err = String; // todo
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        let tag = words.next().unwrap();
        let params = words.map(|word| word.to_string()).collect();

        let command = Command::new(tag, params);

        match command {
            Command::Invalid() => Err("unrecognized command".to_string()),
            _ => Ok(Message {
                prefix: None,
                command,
            }),
        }
    }
}
