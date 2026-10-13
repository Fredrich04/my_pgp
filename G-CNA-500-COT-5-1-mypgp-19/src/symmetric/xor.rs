use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{Plaintext, SymmetricKey};

pub fn xor_block_cipher(message: &Plaintext, key: &SymmetricKey) -> Result<Vec<u8>> {
    ensure_same_size(message.len(), key.len())?;
    Ok(message
        .as_bytes()
        .iter()
        .rev()
        .zip(key.as_bytes())
        .map(|(message_byte, key_byte)| message_byte ^ key_byte)
        .collect())
}

pub fn xor_block_decipher(ciphertext: &[u8], key: &SymmetricKey) -> Result<Plaintext> {
    ensure_same_size(ciphertext.len(), key.len())?;
    let mut message: Vec<u8> = ciphertext
        .iter()
        .zip(key.as_bytes())
        .map(|(cipher_byte, key_byte)| cipher_byte ^ key_byte)
        .collect();
    message.reverse();
    Ok(Plaintext::new(message))
}

fn ensure_same_size(message_bytes: usize, key_bytes: usize) -> Result<()> {
    if message_bytes != key_bytes {
        return Err(MyPgpError::BlockSizeMismatch {
            message_bytes,
            key_bytes,
        });
    }
    Ok(())
}

pub fn xor_stream_cipher(message: &Plaintext, key: &SymmetricKey) -> Result<Vec<u8>> {
    let key_bytes = key.len();
    let mut ciphertext = Vec::with_capacity(message.len().div_ceil(key_bytes) * key_bytes);
    for chunk in message.as_bytes().chunks(key_bytes) {
        let mut block = chunk.to_vec();
        block.resize(key_bytes, 0);
        let block = Plaintext::new(block);
        ciphertext.extend(xor_block_cipher(&block, key)?);
    }
    Ok(ciphertext)
}

pub fn xor_stream_decipher(ciphertext: &[u8], key: &SymmetricKey) -> Result<Plaintext> {
    let key_bytes = key.len();
    if !ciphertext.len().is_multiple_of(key_bytes) {
        return Err(MyPgpError::CiphertextLengthNotMultiple {
            ciphertext_bytes: ciphertext.len(),
            key_bytes,
        });
    }
    let mut message = Vec::with_capacity(ciphertext.len());
    for block in ciphertext.chunks_exact(key_bytes) {
        message.extend_from_slice(xor_block_decipher(block, key)?.as_bytes());
    }
    Ok(Plaintext::new(message))
}
