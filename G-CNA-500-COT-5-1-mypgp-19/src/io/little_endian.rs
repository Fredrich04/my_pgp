use crate::domain::error::{MyPgpError, Result};

const U128_BYTES: usize = 16;

pub fn u128_to_le_bytes(value: u128) -> Vec<u8> {
    let bytes = value.to_le_bytes();
    let significant = bytes
        .iter()
        .rposition(|&byte| byte != 0)
        .map_or(1, |index| index + 1);
    bytes[..significant].to_vec()
}

pub fn le_bytes_to_u128(bytes: &[u8]) -> Result<u128> {
    if bytes.len() > U128_BYTES {
        return Err(MyPgpError::NumberTooLarge {
            max_bytes: U128_BYTES,
        });
    }
    let mut buffer = [0u8; U128_BYTES];
    buffer[..bytes.len()].copy_from_slice(bytes);
    Ok(u128::from_le_bytes(buffer))
}
