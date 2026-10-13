use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::io::little_endian::{le_bytes_to_u128, u128_to_le_bytes};
use my_pgp_lib::io::stdin::{read_message, MAX_INPUT_BYTES};
use my_pgp_lib::Result;
use std::time::{Duration, Instant};

const ONE_MEBIBYTE: usize = 1024 * 1024;
const LIMIT: Duration = Duration::from_secs(2);

fn assert_fast(elapsed: Duration, operation: &str) {
    assert!(
        elapsed < LIMIT,
        "{} took {:?}, limit is {:?} in debug mode (REGLES.md § 34.4)",
        operation,
        elapsed,
        LIMIT
    );
}

#[test]
fn test_one_mebibyte_roundtrip_under_two_seconds() -> Result<()> {
    let bytes: Vec<u8> = (0..ONE_MEBIBYTE).map(|index| (index % 256) as u8).collect();
    let start = Instant::now();
    let decoded = decode_hex(&encode_hex(&bytes))?;
    let elapsed = start.elapsed();
    assert_eq!(
        decoded.len(),
        ONE_MEBIBYTE,
        "the 1 MiB hex roundtrip must keep every byte"
    );
    assert_fast(elapsed, "1 MiB hex roundtrip");
    Ok(())
}

#[test]
fn test_one_million_number_roundtrips_under_two_seconds() -> Result<()> {
    let start = Instant::now();
    for value in 0..1_000_000u128 {
        let number = value.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        assert_eq!(
            le_bytes_to_u128(&u128_to_le_bytes(number))?,
            number,
            "little-endian roundtrip must be exact"
        );
    }
    assert_fast(start.elapsed(), "10^6 little-endian roundtrips");
    Ok(())
}

#[test]
fn test_sixty_four_mebibytes_are_read_under_two_seconds() -> Result<()> {
    let input = std::io::Cursor::new(vec![b'a'; MAX_INPUT_BYTES]);
    let start = Instant::now();
    let message = read_message(input, MAX_INPUT_BYTES)?;
    let elapsed = start.elapsed();
    assert_eq!(
        message.len(),
        MAX_INPUT_BYTES,
        "an input of exactly 64 MiB must be read entirely"
    );
    assert_fast(elapsed, "reading 64 MiB");
    Ok(())
}

#[test]
fn test_one_mebibyte_xor_block_roundtrip_under_two_seconds() -> Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_block_cipher, xor_block_decipher};
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let key = SymmetricKey::new(
        (0..ONE_MEBIBYTE)
            .map(|index| (index * 7 % 256) as u8)
            .collect(),
    )?;
    let message = Plaintext::new((0..ONE_MEBIBYTE).map(|index| (index % 251) as u8).collect());
    let start = Instant::now();
    let deciphered = xor_block_decipher(&xor_block_cipher(&message, &key)?, &key)?;
    let elapsed = start.elapsed();
    assert_eq!(
        deciphered.as_bytes(),
        message.as_bytes(),
        "a 1 MiB XOR block must roundtrip exactly"
    );
    assert_fast(elapsed, "1 MiB XOR block roundtrip");
    Ok(())
}

#[test]
fn test_one_mebibyte_xor_stream_roundtrip_under_two_seconds() -> Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_stream_cipher, xor_stream_decipher};
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let key = SymmetricKey::new((0..16u8).collect())?;
    let message = Plaintext::new((0..ONE_MEBIBYTE).map(|index| (index % 251) as u8).collect());
    let start = Instant::now();
    let deciphered = xor_stream_decipher(&xor_stream_cipher(&message, &key)?, &key)?;
    let elapsed = start.elapsed();
    assert_eq!(
        deciphered.as_bytes(),
        message.as_bytes(),
        "1 MiB is a multiple of 16, so the stream roundtrip must be exact"
    );
    assert_fast(elapsed, "1 MiB XOR stream roundtrip with a 16-byte key");
    Ok(())
}

#[test]
fn test_ten_million_sbox_lookups_under_two_seconds() {
    use my_pgp_lib::symmetric::aes::sbox::{inv_sub_byte, sub_byte};
    let start = Instant::now();
    let mut value = 0u8;
    for _ in 0..10_000_000u32 {
        value = inv_sub_byte(sub_byte(value.wrapping_add(1)));
    }
    assert_fast(start.elapsed(), "10^7 S-box roundtrips");
    assert_eq!(
        value, 128,
        "10^7 increments of a u8 from 0 end at 10^7 mod 256 = 128"
    );
}

#[test]
fn test_hundred_thousand_key_expansions_under_two_seconds() -> Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    use my_pgp_lib::SymmetricKey;
    let key = SymmetricKey::new((0..32u8).collect())?;
    let start = Instant::now();
    for _ in 0..100_000 {
        let keys = expand_key(&key)?;
        assert_eq!(
            keys.rounds(),
            14,
            "every AES-256 expansion must give 14 rounds"
        );
    }
    assert_fast(start.elapsed(), "10^5 AES-256 key expansions");
    Ok(())
}

#[test]
fn test_hundred_thousand_round_trips_under_two_seconds() {
    use my_pgp_lib::symmetric::aes::round::{
        add_round_key, inv_mix_columns, inv_shift_rows, inv_sub_bytes, mix_columns, shift_rows,
        sub_bytes, State,
    };
    let key = [0x2bu8; 16];
    let original: State = core::array::from_fn(|index| index as u8);
    let mut state = original;
    let start = Instant::now();
    for _ in 0..100_000 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &key);
        add_round_key(&mut state, &key);
        inv_mix_columns(&mut state);
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
    }
    assert_fast(start.elapsed(), "10^5 AES round trips");
    assert_eq!(state, original, "every round trip must restore the state");
}
