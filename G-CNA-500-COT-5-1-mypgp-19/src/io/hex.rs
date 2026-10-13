use crate::domain::error::{MyPgpError, Result};

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

pub fn decode_hex(text: &str) -> Result<Vec<u8>> {
    let digits = text.as_bytes();
    if !digits.len().is_multiple_of(2) {
        return Err(MyPgpError::InvalidHexLength {
            digits: digits.len(),
        });
    }
    let mut bytes = Vec::with_capacity(digits.len() / 2);
    for (index, pair) in digits.chunks_exact(2).enumerate() {
        let high = hex_value(pair[0], index * 2)?;
        let low = hex_value(pair[1], index * 2 + 1)?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        text.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn hex_value(digit: u8, position: usize) -> Result<u8> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        b'A'..=b'F' => Ok(digit - b'A' + 10),
        _ => Err(MyPgpError::InvalidHexDigit { position }),
    }
}
