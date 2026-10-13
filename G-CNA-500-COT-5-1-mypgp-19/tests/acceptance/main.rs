use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::io::little_endian::{le_bytes_to_u128, u128_to_le_bytes};
use my_pgp_lib::io::stdin::read_message;
use std::error::Error;
use std::process::Command;

#[test]
fn test_subject_p1_errors_go_to_stderr_with_exit_code_84() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_my_pgp")).output()?;
    assert_eq!(
        output.status.code(),
        Some(84),
        "subject p. 1: exit with 84 on error"
    );
    assert!(
        output.stdout.is_empty(),
        "subject p. 1: errors go to stderr, not stdout; stdout was {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
fn test_subject_examples_print_bare_lowercase_hex() {
    let text = encode_hex(&[0x3b, 0x1f, 0xab]);
    assert_eq!(
        text, "3b1fab",
        "subject examples print lowercase hex without prefix or separator"
    );
}

#[test]
fn test_subject_p4_numbers_are_printed_in_little_endian() -> my_pgp_lib::Result<()> {
    assert_eq!(
        encode_hex(&u128_to_le_bytes(0x0123_4567)),
        "67452301",
        "subject p. 4: 0x01234567 is printed 67452301"
    );
    assert_eq!(
        le_bytes_to_u128(&decode_hex("67452301")?)?,
        0x0123_4567,
        "subject p. 4: 67452301 is read as 0x01234567"
    );
    Ok(())
}

#[test]
fn test_subject_final_newline_is_not_part_of_the_message() -> my_pgp_lib::Result<()> {
    let message = read_message(std::io::Cursor::new(b"All men must die\n".to_vec()), 1024)?;
    assert_eq!(
        message.as_bytes(),
        b"All men must die",
        "the newline added by echo is not part of the message"
    );
    assert_eq!(
        message.len(),
        16,
        "'All men must die' is exactly one 16-byte AES block (subject AES example)"
    );
    Ok(())
}

const SUBJECT_XOR_KEY: &str = "576861742069732064656164206d6179206e6576657220646965";
const SUBJECT_XOR_CIPHERTEXT: &str = "20070f2700071c6a4449060a490515164e4e12190b190011063c";

#[test]
fn test_subject_p4_xor_block_cipher_vector() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_block_cipher;
    use my_pgp_lib::SymmetricKey;
    let message = read_message(
        std::io::Cursor::new(b"You know nothing, Jon Snow\n".to_vec()),
        1024,
    )?;
    let key = SymmetricKey::new(decode_hex(SUBJECT_XOR_KEY)?)?;
    assert_eq!(
        encode_hex(&xor_block_cipher(&message, &key)?),
        SUBJECT_XOR_CIPHERTEXT,
        "subject p. 4: xor -c -b must print this exact ciphertext"
    );
    Ok(())
}

#[test]
fn test_subject_p4_xor_block_decipher_vector() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_block_decipher;
    use my_pgp_lib::SymmetricKey;
    let key = SymmetricKey::new(decode_hex(SUBJECT_XOR_KEY)?)?;
    let message = xor_block_decipher(&decode_hex(SUBJECT_XOR_CIPHERTEXT)?, &key)?;
    assert_eq!(
        message.as_bytes(),
        b"You know nothing, Jon Snow",
        "subject p. 4: xor -d -b must give the original message back"
    );
    Ok(())
}

#[test]
fn test_subject_p4_xor_stream_matches_block_on_one_block() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_stream_cipher;
    use my_pgp_lib::SymmetricKey;
    let message = read_message(
        std::io::Cursor::new(b"You know nothing, Jon Snow\n".to_vec()),
        1024,
    )?;
    let key = SymmetricKey::new(decode_hex(SUBJECT_XOR_KEY)?)?;
    assert_eq!(
        encode_hex(&xor_stream_cipher(&message, &key)?),
        SUBJECT_XOR_CIPHERTEXT,
        "subject p. 4: a one-block message gives the official output in stream mode too"
    );
    Ok(())
}

#[test]
fn test_subject_p4_xor_stream_pads_last_block_with_zeros() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_stream_cipher, xor_stream_decipher};
    use my_pgp_lib::SymmetricKey;
    let message = read_message(std::io::Cursor::new(b"Hodor\n".to_vec()), 1024)?;
    let key = SymmetricKey::new(decode_hex("0102030405060708")?)?;
    let ciphertext = xor_stream_cipher(&message, &key)?;
    assert_eq!(
        ciphertext.len(),
        8,
        "subject p. 4: a 5-byte message is padded to one 8-byte block"
    );
    assert_eq!(
        xor_stream_decipher(&ciphertext, &key)?.as_bytes(),
        b"Hodor\0\0\0",
        "the zero padding is visible after decipher"
    );
    Ok(())
}

#[test]
fn test_subject_p5_aes_sbox_is_the_standard_one() {
    use my_pgp_lib::symmetric::aes::sbox::sub_byte;
    assert_eq!(
        sub_byte(0x53),
        0xed,
        "subject p. 5 requires AES; its S-box must be the FIPS-197 one (S(53) = ed)"
    );
}

#[test]
fn test_subject_p5_aes_accepts_128_192_and_256_bit_keys() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    use my_pgp_lib::SymmetricKey;
    for (bytes, rounds) in [(16, 10), (24, 12), (32, 14)] {
        let keys = expand_key(&SymmetricKey::new(vec![0x42; bytes])?)?;
        assert_eq!(
            keys.rounds(),
            rounds,
            "subject p. 5: AES-128 is required, AES-192/256 expected; {}-byte keys must work",
            bytes
        );
    }
    Ok(())
}

#[test]
fn test_subject_p5_aes_round_uses_the_standard_mix_columns() {
    use my_pgp_lib::symmetric::aes::round::{mix_columns, State};
    let mut state: State = [
        0xdb, 0x13, 0x53, 0x45, 0, 0, 0, 0, 1, 1, 1, 1, 0xc6, 0xc6, 0xc6, 0xc6,
    ];
    mix_columns(&mut state);
    assert_eq!(
        state,
        [0x8e, 0x4d, 0xa1, 0xbc, 0, 0, 0, 0, 1, 1, 1, 1, 0xc6, 0xc6, 0xc6, 0xc6],
        "subject p. 5 requires AES; MixColumns must be the FIPS-197 one, got {:02x?}",
        state
    );
}
