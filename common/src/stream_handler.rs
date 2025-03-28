use std::io::{self, BufRead};

use crate::{IrcError, Message};

pub fn blocking_read_until_cr_or_lf<R: BufRead>(
    reader: &mut R,
    buffer: &mut String,
) -> io::Result<Option<String>> {
    loop {
        if let Some(pos_rn) = buffer.find(['\r', '\n']) {
            let msg = buffer[..pos_rn].to_string();
            *buffer = buffer[pos_rn + 1..].to_string();
            if !msg.is_empty() {
                return Ok(Some(msg));
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
