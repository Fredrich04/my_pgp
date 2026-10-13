use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{wipe, SymmetricKey};
use crate::symmetric::aes::gf256::gf_mul;
use crate::symmetric::aes::sbox::sub_byte;
use std::fmt;

pub const BLOCK_BYTES: usize = 16;
const WORD_BYTES: usize = 4;

pub struct RoundKeys {
    bytes: Vec<u8>,
    rounds: usize,
}

impl RoundKeys {
    pub fn rounds(&self) -> usize {
        self.rounds
    }

    pub fn round_key(&self, round: usize) -> Option<&[u8]> {
        self.bytes.chunks_exact(BLOCK_BYTES).nth(round)
    }
}

impl fmt::Debug for RoundKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RoundKeys(<{} rounds, redacted>)", self.rounds)
    }
}

impl Drop for RoundKeys {
    fn drop(&mut self) {
        wipe(&mut self.bytes);
    }
}

pub fn expand_key(key: &SymmetricKey) -> Result<RoundKeys> {
    let key_words = match key.len() {
        16 => 4,
        24 => 6,
        32 => 8,
        bytes => return Err(MyPgpError::InvalidKeyLength { bytes }),
    };
    let rounds = key_words + 6;
    let total_words = (rounds + 1) * (BLOCK_BYTES / WORD_BYTES);
    let mut bytes = Vec::with_capacity(total_words * WORD_BYTES);
    bytes.extend_from_slice(key.as_bytes());
    let mut round_constant = 1u8;
    let mut temp = [0u8; WORD_BYTES];
    for word in key_words..total_words {
        temp.copy_from_slice(&bytes[(word - 1) * WORD_BYTES..word * WORD_BYTES]);
        if word % key_words == 0 {
            temp.rotate_left(1);
            temp.iter_mut().for_each(|byte| *byte = sub_byte(*byte));
            temp[0] ^= round_constant;
            round_constant = gf_mul(round_constant, 2);
        } else if key_words > 6 && word % key_words == 4 {
            temp.iter_mut().for_each(|byte| *byte = sub_byte(*byte));
        }
        let previous = (word - key_words) * WORD_BYTES;
        for offset in 0..WORD_BYTES {
            bytes.push(bytes[previous + offset] ^ temp[offset]);
        }
    }
    wipe(&mut temp);
    Ok(RoundKeys { bytes, rounds })
}
