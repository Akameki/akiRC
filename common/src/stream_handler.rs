use std::io::{self, BufRead};

use crate::{IrcError, message::Message};

pub fn blocking_read_until_cr_or_lf<R: BufRead>(
    reader: &mut R,
    buffer: &mut String,
) -> io::Result<Option<String>> {
    loop {
        if let Some(pos_rn) = buffer.find(['\r', '\n']) {
            let line = buffer[..pos_rn].to_owned();
            *buffer = buffer[pos_rn + 1..].to_owned();
            if !line.is_empty() {
                println!("rec < {line}");
                return Ok(Some(line));
            }
        } else if reader.read_line(buffer)? == 0 {
            return Ok(None); // EOF
        }
    }
}

pub fn blocking_read_message<R: BufRead>(
    reader: &mut R,
    buffer: &mut String,
) -> Result<Message, IrcError> {
    match blocking_read_until_cr_or_lf(reader, buffer) {
        Ok(Some(s)) => s.parse(),
        Ok(None) => Err(IrcError::Eof),
        Err(e) => Err(e.into()),
    }
}
