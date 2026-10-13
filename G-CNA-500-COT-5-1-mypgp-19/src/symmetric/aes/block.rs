use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{Plaintext, SymmetricKey};
use crate::symmetric::aes::key_schedule::{expand_key, RoundKeys, BLOCK_BYTES};
use crate::symmetric::aes::round::{
    add_round_key, inv_mix_columns, inv_shift_rows, inv_sub_bytes, mix_columns, shift_rows,
    sub_bytes, State,
};

const WORD_BYTES: usize = 4;

pub fn encrypt_block(message: &Plaintext, key: &SymmetricKey) -> Result<Vec<u8>> {
    ensure_block_size(message.len())?;
    let round_keys = subject_round_keys(key)?;
    let mut state: State = [0u8; BLOCK_BYTES];
    state.copy_from_slice(message.as_bytes());
    cipher_rounds(&mut state, &round_keys);
    reverse_words(&mut state);
    Ok(state.to_vec())
}

pub fn decrypt_block(ciphertext: &[u8], key: &SymmetricKey) -> Result<Plaintext> {
    ensure_block_size(ciphertext.len())?;
    let round_keys = subject_round_keys(key)?;
    let mut state: State = [0u8; BLOCK_BYTES];
    state.copy_from_slice(ciphertext);
    reverse_words(&mut state);
    inv_cipher_rounds(&mut state, &round_keys);
    Ok(Plaintext::new(state.to_vec()))
}

fn subject_round_keys(key: &SymmetricKey) -> Result<RoundKeys> {
    let mut fips_key_bytes = key.as_bytes().to_vec();
    reverse_words(&mut fips_key_bytes);
    let fips_key = SymmetricKey::new(fips_key_bytes)?;
    expand_key(&fips_key)
}

fn cipher_rounds(state: &mut State, round_keys: &RoundKeys) {
    let last_round = round_keys.rounds();
    add_round_key(state, round_keys.round_key(0).unwrap_or_default());
    for round in 1..last_round {
        sub_bytes(state);
        shift_rows(state);
        mix_columns(state);
        add_round_key(state, round_keys.round_key(round).unwrap_or_default());
    }
    sub_bytes(state);
    shift_rows(state);
    add_round_key(state, round_keys.round_key(last_round).unwrap_or_default());
}

fn inv_cipher_rounds(state: &mut State, round_keys: &RoundKeys) {
    let last_round = round_keys.rounds();
    add_round_key(state, round_keys.round_key(last_round).unwrap_or_default());
    for round in (1..last_round).rev() {
        inv_shift_rows(state);
        inv_sub_bytes(state);
        add_round_key(state, round_keys.round_key(round).unwrap_or_default());
        inv_mix_columns(state);
    }
    inv_shift_rows(state);
    inv_sub_bytes(state);
    add_round_key(state, round_keys.round_key(0).unwrap_or_default());
}

fn reverse_words(bytes: &mut [u8]) {
    bytes
        .chunks_exact_mut(WORD_BYTES)
        .for_each(|word| word.reverse());
}

fn ensure_block_size(message_bytes: usize) -> Result<()> {
    if message_bytes != BLOCK_BYTES {
        return Err(MyPgpError::BlockSizeMismatch {
            message_bytes,
            key_bytes: BLOCK_BYTES,
        });
    }
    Ok(())
}
