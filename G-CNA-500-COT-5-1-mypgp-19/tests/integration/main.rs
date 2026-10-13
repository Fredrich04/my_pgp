use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::io::little_endian::{le_bytes_to_u128, u128_to_le_bytes};
use my_pgp_lib::io::stdin::read_message;
use my_pgp_lib::{Result, SymmetricKey};

#[test]
fn test_decoded_hex_builds_a_symmetric_key() -> Result<()> {
    let key = SymmetricKey::new(decode_hex("000102030405060708090a0b0c0d0e0f")?)?;
    assert_eq!(
        key.len(),
        16,
        "32 hex digits must give a 16-byte AES-128 key"
    );
    assert_eq!(
        encode_hex(key.as_bytes()),
        "000102030405060708090a0b0c0d0e0f",
        "the key must hold exactly the decoded bytes"
    );
    Ok(())
}

#[test]
fn test_empty_hex_cannot_become_a_key() -> Result<()> {
    let bytes = decode_hex("")?;
    let result = SymmetricKey::new(bytes);
    assert!(
        result.is_err(),
        "an empty hex key must be rejected by SymmetricKey (REGLES.md § 1.3), got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_roundtrip_over_every_byte_value() -> Result<()> {
    let bytes: Vec<u8> = (0..=255).collect();
    assert_eq!(
        decode_hex(&encode_hex(&bytes))?,
        bytes,
        "decode(encode(b)) must give b back for all 256 byte values"
    );
    Ok(())
}

#[test]
fn test_number_to_hex_follows_subject_convention() -> Result<()> {
    assert_eq!(
        encode_hex(&u128_to_le_bytes(0x0123_4567)),
        "67452301",
        "number → little-endian bytes → hex must match the subject (p. 4)"
    );
    assert_eq!(
        le_bytes_to_u128(&decode_hex("67452301")?)?,
        0x0123_4567,
        "hex → bytes → number must match the subject (p. 4)"
    );
    Ok(())
}

#[test]
fn test_hex_message_read_from_input_is_decoded(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let input = std::io::Cursor::new(b"67452301\n".to_vec());
    let message = read_message(input, 64)?;
    let text = std::str::from_utf8(message.as_bytes())?;
    assert_eq!(
        decode_hex(text)?,
        vec![0x67, 0x45, 0x23, 0x01],
        "a hex line read from stdin must decode once its newline is stripped"
    );
    Ok(())
}

#[test]
fn test_xor_block_from_stdin_and_hex_key() -> Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_block_cipher, xor_block_decipher};
    let message = read_message(std::io::Cursor::new(b"Hi!\n".to_vec()), 64)?;
    let key = SymmetricKey::new(decode_hex("0a0b0c")?)?;
    let ciphertext = encode_hex(&xor_block_cipher(&message, &key)?);
    let deciphered = xor_block_decipher(&decode_hex(&ciphertext)?, &key)?;
    assert_eq!(
        deciphered.as_bytes(),
        b"Hi!",
        "stdin → XOR -b → hex → XOR -b must give the message back, ciphertext was {}",
        ciphertext
    );
    Ok(())
}

#[test]
fn test_xor_stream_from_stdin_and_hex_key() -> Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_stream_cipher, xor_stream_decipher};
    let message = read_message(std::io::Cursor::new(b"Winter is coming\n".to_vec()), 64)?;
    let key = SymmetricKey::new(decode_hex("0a0b0c")?)?;
    let ciphertext = encode_hex(&xor_stream_cipher(&message, &key)?);
    let deciphered = xor_stream_decipher(&decode_hex(&ciphertext)?, &key)?;
    assert_eq!(
        deciphered.as_bytes(),
        b"Winter is coming\0\0",
        "16 bytes with a 3-byte key give 6 blocks; the 2 padding zeros stay after decipher, ciphertext was {}",
        ciphertext
    );
    Ok(())
}

#[test]
fn test_sbox_is_built_from_gf_inverse_and_affine_map() {
    use my_pgp_lib::symmetric::aes::gf256::gf_inverse;
    use my_pgp_lib::symmetric::aes::sbox::sub_byte;
    for value in 0..=255u8 {
        let inverse = gf_inverse(value);
        let affine = inverse
            ^ inverse.rotate_left(1)
            ^ inverse.rotate_left(2)
            ^ inverse.rotate_left(3)
            ^ inverse.rotate_left(4)
            ^ 0x63;
        assert_eq!(
            sub_byte(value),
            affine,
            "the S-box must equal the FIPS-197 § 5.1.1 construction for {:#04x}",
            value
        );
    }
}

#[test]
fn test_hex_key_from_the_subject_expands_to_eleven_round_keys() -> Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    let key = SymmetricKey::new(decode_hex("57696e74657220697320636f6d696e67")?)?;
    let keys = expand_key(&key)?;
    let count = (0..=keys.rounds())
        .filter(|&round| keys.round_key(round).is_some())
        .count();
    assert_eq!(
        count, 11,
        "hex key → SymmetricKey → schedule must give 11 AES-128 round keys"
    );
    Ok(())
}

#[test]
fn test_expanded_key_and_round_steps_reproduce_fips_197_round_one() -> Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    use my_pgp_lib::symmetric::aes::round::{
        add_round_key, mix_columns, shift_rows, sub_bytes, State,
    };
    let keys = expand_key(&SymmetricKey::new(decode_hex(
        "2b7e151628aed2a6abf7158809cf4f3c",
    )?)?)?;
    let mut state: State = [0u8; 16];
    state.copy_from_slice(&decode_hex("3243f6a8885a308d313198a2e0370734")?);
    add_round_key(&mut state, keys.round_key(0).unwrap_or_default());
    sub_bytes(&mut state);
    shift_rows(&mut state);
    mix_columns(&mut state);
    add_round_key(&mut state, keys.round_key(1).unwrap_or_default());
    assert_eq!(
        encode_hex(&state),
        "a49c7ff2689f352b6b5bea43026a5049",
        "FIPS-197 appendix B: key schedule + round 1 must give the start of round 2"
    );
    Ok(())
}
