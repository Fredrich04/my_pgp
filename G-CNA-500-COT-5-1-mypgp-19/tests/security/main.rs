use my_pgp_lib::io::hex::decode_hex;
use my_pgp_lib::io::little_endian::le_bytes_to_u128;
use my_pgp_lib::io::stdin::read_message;
use my_pgp_lib::{MyPgpError, SymmetricKey};

fn assert_no_leak(text: &str, secrets: &[&str], context: &str) {
    for secret in secrets {
        assert!(
            !text.contains(secret),
            "{} leaks '{}' (REGLES.md § 4.4, § 4.5, BLOCKER-13): {}",
            context,
            secret,
            text
        );
    }
}

#[test]
fn test_decode_error_never_echoes_the_input() {
    let result = decode_hex("deadbeefcafeZZ");
    assert!(result.is_err(), "'ZZ' must be rejected, got {:?}", result);
    if let Err(error) = result {
        assert_no_leak(
            &error.to_string(),
            &["dead", "cafe", "ZZ"],
            "a hex error message",
        );
    }
}

#[test]
fn test_decode_survives_every_single_byte_input_without_panicking() {
    for byte in 0u8..=127 {
        let text = String::from(char::from(byte)).repeat(2);
        let result = decode_hex(&text);
        assert_eq!(
            result.is_ok(),
            byte.is_ascii_hexdigit(),
            "byte {:#04x} doubled must be accepted exactly when it is a hex digit, got {:?}",
            byte,
            result
        );
    }
}

#[test]
fn test_decode_survives_hostile_unicode() {
    for text in ["\u{0}\u{0}", "٣٣", "🔑🔑", "\u{202e}ab", "ab\ncd"] {
        let result = decode_hex(text);
        assert!(
            result.is_err(),
            "hostile input {:?} must be rejected without panic (REGLES.md § 17), got {:?}",
            text,
            result
        );
    }
}

#[test]
fn test_decoded_key_debug_does_not_leak() -> my_pgp_lib::Result<()> {
    let key = SymmetricKey::new(decode_hex("73656372657421")?)?;
    assert_no_leak(
        &format!("{:?}", key),
        &["7365", "secret"],
        "SymmetricKey Debug",
    );
    Ok(())
}

#[test]
fn test_oversized_number_is_rejected_without_panic_or_echo() {
    for length in 17..=64 {
        let result = le_bytes_to_u128(&vec![0xabu8; length]);
        assert!(
            result.is_err(),
            "{} bytes exceed a u128 and must be rejected, got {:?}",
            length,
            result
        );
        if let Err(error) = result {
            assert_no_leak(&error.to_string(), &["ab"], "a number size error");
        }
    }
}

#[test]
fn test_endless_input_is_cut_at_the_limit() {
    let result = read_message(std::io::repeat(b'x'), 1024);
    assert!(
        matches!(result, Err(MyPgpError::InputTooLarge { limit_bytes: 1024 })),
        "an endless stdin must stop at the limit instead of exhausting memory (BLOCKER-16), got {:?}",
        result
    );
}

#[test]
fn test_oversized_input_error_does_not_echo_content() {
    let result = read_message(std::io::Cursor::new(b"topsecret".to_vec()), 4);
    assert!(
        result.is_err(),
        "9 bytes exceed a 4-byte limit, got {:?}",
        result
    );
    if let Err(error) = result {
        assert_no_leak(
            &error.to_string(),
            &["top", "secret"],
            "an input size error",
        );
    }
}

#[test]
fn test_read_message_debug_does_not_leak_plaintext() -> my_pgp_lib::Result<()> {
    let message = read_message(std::io::Cursor::new(b"attack at dawn\n".to_vec()), 64)?;
    assert_no_leak(
        &format!("{:?}", message),
        &["attack", "dawn"],
        "Plaintext Debug",
    );
    Ok(())
}

#[test]
fn test_xor_size_error_does_not_leak_key_or_message() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_block_cipher;
    use my_pgp_lib::Plaintext;
    let key = SymmetricKey::new(b"hunter2".to_vec())?;
    let result = xor_block_cipher(&Plaintext::new(b"launch codes".to_vec()), &key);
    assert!(
        result.is_err(),
        "sizes differ, the block must be rejected, got {:?}",
        result
    );
    if let Err(error) = result {
        assert_no_leak(
            &error.to_string(),
            &["hunter", "launch", "codes"],
            "a block size error",
        );
    }
    Ok(())
}

#[test]
fn test_xor_deciphered_message_debug_is_redacted() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_block_cipher, xor_block_decipher};
    use my_pgp_lib::Plaintext;
    let key = SymmetricKey::new(vec![0x42; 6])?;
    let ciphertext = xor_block_cipher(&Plaintext::new(b"secret".to_vec()), &key)?;
    let message = xor_block_decipher(&ciphertext, &key)?;
    assert_no_leak(
        &format!("{:?}", message),
        &["secret"],
        "deciphered Plaintext Debug",
    );
    Ok(())
}

#[test]
fn test_xor_stream_length_error_does_not_leak() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_stream_decipher;
    let key = SymmetricKey::new(b"hunter2".to_vec())?;
    let result = xor_stream_decipher(b"leak", &key);
    assert!(
        result.is_err(),
        "4 bytes are not a multiple of 7, got {:?}",
        result
    );
    if let Err(error) = result {
        assert_no_leak(
            &error.to_string(),
            &["hunter", "leak"],
            "a stream length error",
        );
    }
    Ok(())
}

#[test]
fn test_xor_stream_survives_every_key_size_up_to_message_length() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_stream_cipher, xor_stream_decipher};
    use my_pgp_lib::Plaintext;
    let original = b"The night is dark and full of terrors".to_vec();
    for size in 1..=original.len() + 3 {
        let key = SymmetricKey::new(vec![0x5a; size])?;
        let ciphertext = xor_stream_cipher(&Plaintext::new(original.clone()), &key)?;
        let message = xor_stream_decipher(&ciphertext, &key)?;
        assert!(
            message.as_bytes().starts_with(&original)
                && message.as_bytes()[original.len()..]
                    .iter()
                    .all(|&byte| byte == 0),
            "key size {} must roundtrip up to zero padding, got {:?}",
            size,
            message
        );
    }
    Ok(())
}

#[test]
fn test_sbox_is_a_permutation_without_fixed_points() {
    use my_pgp_lib::symmetric::aes::sbox::SBOX;
    let mut seen = [false; 256];
    for (input, &output) in SBOX.iter().enumerate() {
        assert!(
            !seen[usize::from(output)],
            "S-box output {:#04x} appears twice",
            output
        );
        seen[usize::from(output)] = true;
        assert_ne!(
            usize::from(output),
            input,
            "S-box must have no fixed point, {:#04x}",
            input
        );
        assert_ne!(
            usize::from(output),
            input ^ 0xff,
            "S-box must have no opposite fixed point, {:#04x}",
            input
        );
    }
}

#[test]
fn test_round_keys_debug_is_redacted() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    let keys = expand_key(&SymmetricKey::new(decode_hex(
        "2b7e151628aed2a6abf7158809cf4f3c",
    )?)?)?;
    let debug = format!("{:?}", keys);
    assert_eq!(
        debug, "RoundKeys(<10 rounds, redacted>)",
        "round keys are secret (REGLES.md § 4.1)"
    );
    assert_no_leak(&debug, &["2b7e", "a0fa", "d014"], "RoundKeys Debug");
    Ok(())
}

#[test]
fn test_one_bit_key_change_changes_the_last_round_key_widely() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    let first = expand_key(&SymmetricKey::new(vec![0u8; 16])?)?;
    let mut flipped = vec![0u8; 16];
    flipped[0] = 1;
    let second = expand_key(&SymmetricKey::new(flipped)?)?;
    let differing_bits: u32 = match (first.round_key(10), second.round_key(10)) {
        (Some(left), Some(right)) => left
            .iter()
            .zip(right)
            .map(|(a, b)| (a ^ b).count_ones())
            .sum(),
        _ => 0,
    };
    assert!(
        differing_bits >= 32,
        "one flipped key bit must spread to many bits of the last round key, only {} of 128 differ",
        differing_bits
    );
    Ok(())
}

#[test]
fn test_invalid_aes_key_error_does_not_leak() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    let result = expand_key(&SymmetricKey::new(b"hunter2".to_vec())?);
    assert!(
        result.is_err(),
        "a 7-byte AES key must be rejected, got {:?}",
        result
    );
    if let Err(error) = result {
        assert_no_leak(&error.to_string(), &["hunter"], "an AES key size error");
    }
    Ok(())
}

#[test]
fn test_two_rounds_spread_one_byte_to_the_whole_state() {
    use my_pgp_lib::symmetric::aes::round::{mix_columns, shift_rows, sub_bytes, State};
    let run = |state: &mut State| {
        for _ in 0..2 {
            sub_bytes(state);
            shift_rows(state);
            mix_columns(state);
        }
    };
    let mut first: State = [0u8; 16];
    let mut second: State = [0u8; 16];
    second[0] = 1;
    run(&mut first);
    run(&mut second);
    let changed = first.iter().zip(&second).filter(|(a, b)| a != b).count();
    assert_eq!(
        changed, 16,
        "FIPS-197 diffusion: two rounds must spread one byte to all 16, only {} changed",
        changed
    );
}
