use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::io::little_endian::{le_bytes_to_u128, u128_to_le_bytes};
use my_pgp_lib::io::stdin::read_message;
use my_pgp_lib::{MyPgpError, Result};

#[test]
fn test_frozen_encoding_of_every_byte_value() {
    let bytes: Vec<u8> = (0..=255).collect();
    let text = encode_hex(&bytes);
    assert!(
        text.starts_with("000102030405060708090a0b0c0d0e0f10"),
        "frozen prefix of the 256-byte encoding changed: {}",
        &text[..34]
    );
    assert!(
        text.ends_with("f8f9fafbfcfdfeff"),
        "frozen suffix of the 256-byte encoding changed: {}",
        &text[text.len() - 16..]
    );
}

#[test]
fn test_frozen_subject_byte_order_example() -> Result<()> {
    assert_eq!(
        decode_hex("67452301")?,
        vec![0x67, 0x45, 0x23, 0x01],
        "frozen: hex text decodes in text order"
    );
    assert_eq!(
        encode_hex(&[0x67, 0x45, 0x23, 0x01]),
        "67452301",
        "frozen: bytes encode in byte order"
    );
    Ok(())
}

#[test]
fn test_frozen_non_ascii_rejection_order() {
    let odd = decode_hex("é0");
    assert!(
        matches!(odd, Err(MyPgpError::InvalidHexLength { digits: 3 })),
        "frozen: 'é0' is 3 bytes, so the length check fires first, got {:?}",
        odd
    );
    let even = decode_hex("é00");
    assert!(
        matches!(even, Err(MyPgpError::InvalidHexDigit { position: 0 })),
        "frozen: 'é00' is 4 bytes, so the digit check fires at position 0, got {:?}",
        even
    );
}

#[test]
fn test_frozen_little_endian_vectors() -> Result<()> {
    let cases: [(u128, &str); 5] = [
        (0, "00"),
        (1, "01"),
        (0x0123_4567, "67452301"),
        (0xd3, "d3"),
        (
            0x0100_0000_0000_0000_0000_0000_0000_0000,
            "00000000000000000000000000000001",
        ),
    ];
    for (value, hex) in cases {
        assert_eq!(
            encode_hex(&u128_to_le_bytes(value)),
            hex,
            "frozen little-endian encoding of {:#x}",
            value
        );
        assert_eq!(
            le_bytes_to_u128(&decode_hex(hex)?)?,
            value,
            "frozen little-endian decoding of {}",
            hex
        );
    }
    Ok(())
}

#[test]
fn test_frozen_newline_stripping_rules() -> Result<()> {
    let cases: [(&[u8], &[u8]); 6] = [
        (b"Winter is coming\n", b"Winter is coming"),
        (b"Winter is coming\r\n", b"Winter is coming"),
        (b"Winter is coming", b"Winter is coming"),
        (b"a\n\n", b"a\n"),
        (b"a\r", b"a\r"),
        (b"", b""),
    ];
    for (input, expected) in cases {
        let message = read_message(std::io::Cursor::new(input.to_vec()), 64)?;
        assert_eq!(
            message.as_bytes(),
            expected,
            "frozen newline rule for input {:?}",
            String::from_utf8_lossy(input)
        );
    }
    Ok(())
}

#[test]
fn test_frozen_xor_block_vectors() -> Result<()> {
    use my_pgp_lib::symmetric::xor::xor_block_cipher;
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let cases: [(&[u8], &str, &str); 3] = [
        (b"A", "01", "40"),
        (b"AB", "0102", "4343"),
        (b"abcd", "00000000", "64636261"),
    ];
    for (message, key_hex, expected) in cases {
        let key = SymmetricKey::new(decode_hex(key_hex)?)?;
        let ciphertext = xor_block_cipher(&Plaintext::new(message.to_vec()), &key)?;
        assert_eq!(
            encode_hex(&ciphertext),
            expected,
            "frozen XOR -b output for {:?} with key {}",
            String::from_utf8_lossy(message),
            key_hex
        );
    }
    Ok(())
}

#[test]
fn test_frozen_xor_stream_vectors() -> Result<()> {
    use my_pgp_lib::symmetric::xor::xor_stream_cipher;
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let cases: [(&[u8], &str, &str); 3] = [
        (b"abc", "0102", "63630161"),
        (b"", "01", ""),
        (b"abcd", "00", "61626364"),
    ];
    for (message, key_hex, expected) in cases {
        let key = SymmetricKey::new(decode_hex(key_hex)?)?;
        let ciphertext = xor_stream_cipher(&Plaintext::new(message.to_vec()), &key)?;
        assert_eq!(
            encode_hex(&ciphertext),
            expected,
            "frozen XOR stream output for {:?} with key {}",
            String::from_utf8_lossy(message),
            key_hex
        );
    }
    Ok(())
}

#[test]
fn test_frozen_sbox_rows() {
    use my_pgp_lib::symmetric::aes::sbox::{INV_SBOX, SBOX};
    assert_eq!(
        encode_hex(&SBOX[..16]),
        "637c777bf26b6fc53001672bfed7ab76",
        "frozen first row of the FIPS-197 S-box"
    );
    assert_eq!(
        encode_hex(&SBOX[240..]),
        "8ca1890dbfe6426841992d0fb054bb16",
        "frozen last row of the FIPS-197 S-box"
    );
    assert_eq!(
        encode_hex(&INV_SBOX[..16]),
        "52096ad53036a538bf40a39e81f3d7fb",
        "frozen first row of the FIPS-197 inverse S-box"
    );
}

#[test]
fn test_frozen_aes_round_key_five() -> Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    use my_pgp_lib::SymmetricKey;
    let cases = [
        (
            "2b7e151628aed2a6abf7158809cf4f3c",
            "d4d1c6f87c839d87caf2b8bc11f915bc",
        ),
        (
            "8e73b0f7da0e6452c810f32b809079e562f8ead2522c6b7b",
            "a448f6d94d6dce24aa326360113b30e6",
        ),
        (
            "603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4",
            "b5a9328a2678a647983122292f6c79b3",
        ),
    ];
    for (key_hex, expected) in cases {
        let keys = expand_key(&SymmetricKey::new(decode_hex(key_hex)?)?)?;
        assert_eq!(
            keys.round_key(5).map(encode_hex),
            Some(expected.to_string()),
            "frozen round key 5 for the FIPS-197 appendix A key {}",
            key_hex
        );
    }
    Ok(())
}

#[test]
fn test_frozen_mix_columns_of_counting_state() {
    use my_pgp_lib::symmetric::aes::round::{mix_columns, State};
    let mut state: State = core::array::from_fn(|index| index as u8);
    mix_columns(&mut state);
    assert_eq!(
        encode_hex(&state),
        "02070005060304010a0f080d0e0b0c09",
        "frozen MixColumns of 00..0f, computed independently"
    );
}
