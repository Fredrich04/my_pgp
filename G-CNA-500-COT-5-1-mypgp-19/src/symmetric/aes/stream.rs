use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{Plaintext, SymmetricKey};
use crate::symmetric::aes::block::{decrypt_block, encrypt_block};
use crate::symmetric::aes::key_schedule::BLOCK_BYTES;

pub fn aes_stream_cipher(message: &Plaintext, key: &SymmetricKey) -> Result<Vec<u8>> {
    let mut ciphertext = Vec::with_capacity(padded_len(message.len()));
    for chunk in message.as_bytes().chunks(BLOCK_BYTES) {
        let mut block_bytes = chunk.to_vec();
        block_bytes.resize(BLOCK_BYTES, 0);
        let block = Plaintext::new(block_bytes);
        ciphertext.extend(encrypt_block(&block, key)?);
    }
    Ok(ciphertext)
}

pub fn aes_stream_decipher(ciphertext: &[u8], key: &SymmetricKey) -> Result<Plaintext> {
    if !ciphertext.len().is_multiple_of(BLOCK_BYTES) {
        return Err(MyPgpError::CiphertextLengthNotMultiple {
            ciphertext_bytes: ciphertext.len(),
            key_bytes: BLOCK_BYTES,
        });
    }
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    for chunk in ciphertext.chunks_exact(BLOCK_BYTES) {
        plaintext.extend_from_slice(decrypt_block(chunk, key)?.as_bytes());
    }
    Ok(Plaintext::new(plaintext))
}

fn padded_len(len: usize) -> usize {
    if len == 0 {
        BLOCK_BYTES
    } else {
        len.div_ceil(BLOCK_BYTES) * BLOCK_BYTES
    }
}
