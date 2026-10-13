use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{wipe, Plaintext};
use std::io::Read;

pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

pub fn read_message<R: Read>(reader: R, limit_bytes: usize) -> Result<Plaintext> {
    let allowed = u64::try_from(limit_bytes)
        .map_err(|_| MyPgpError::InputTooLarge { limit_bytes })?
        .saturating_add(1);
    let mut message = Vec::new();
    if let Err(source) = reader.take(allowed).read_to_end(&mut message) {
        wipe(&mut message);
        return Err(MyPgpError::from(source));
    }
    if message.len() > limit_bytes {
        wipe(&mut message);
        return Err(MyPgpError::InputTooLarge { limit_bytes });
    }
    strip_final_newline(&mut message);
    Ok(Plaintext::new(message))
}

fn strip_final_newline(message: &mut Vec<u8>) {
    if message.ends_with(b"\r\n") {
        message.truncate(message.len() - 2);
    } else if message.ends_with(b"\n") {
        message.truncate(message.len() - 1);
    }
}
