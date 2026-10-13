use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::io::little_endian::u128_to_le_bytes;
use my_pgp_lib::io::stdin::read_message;
use my_pgp_lib::MyPgpError;
use std::error::Error;
use std::process::Command;

const ESCAPE: char = '\u{1b}';

fn assert_screen_reader_friendly(text: &str, context: &str) {
    assert!(
        text.is_ascii(),
        "{} must be plain ASCII for screen readers and tools: {:?}",
        context,
        text
    );
    assert!(
        !text.contains(ESCAPE),
        "{} must not contain color or escape sequences: {:?}",
        context,
        text
    );
    assert!(
        !text.contains('\n'),
        "{} must hold on a single line: {:?}",
        context,
        text
    );
}

fn error_text<T: std::fmt::Debug>(result: Result<T, MyPgpError>) -> String {
    match result {
        Err(error) => error.to_string(),
        Ok(value) => format!("<unexpected success: {:?}>", value),
    }
}

#[test]
fn test_hex_output_is_plain_single_line_ascii() {
    let text = encode_hex(&(0..=255).collect::<Vec<u8>>());
    assert_screen_reader_friendly(&text, "hex output");
}

#[test]
fn test_error_messages_are_plain_ascii_without_color_codes() {
    for input in ["abc", "zz"] {
        let result = decode_hex(input);
        assert!(
            result.is_err(),
            "{:?} must be rejected, got {:?}",
            input,
            result
        );
        assert_screen_reader_friendly(&error_text(result), "a hex error message");
    }
}

#[test]
fn test_binary_error_output_has_no_escape_sequences() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_my_pgp")).output()?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_screen_reader_friendly(stderr.trim_end(), "the binary error output");
    Ok(())
}

#[test]
fn test_little_endian_hex_output_is_plain_ascii() {
    let text = encode_hex(&u128_to_le_bytes(u128::MAX));
    assert_screen_reader_friendly(&text, "a little-endian number");
    assert_eq!(text.len(), 32, "u128::MAX must print as exactly 32 digits");
}

#[test]
fn test_input_error_message_is_plain_ascii() {
    let result = read_message(std::io::Cursor::new(vec![0u8; 10]), 8);
    assert!(
        result.is_err(),
        "10 bytes exceed an 8-byte limit, got {:?}",
        result
    );
    assert_screen_reader_friendly(&error_text(result), "an input error message");
}

#[test]
fn test_xor_output_and_errors_are_plain_ascii() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_block_cipher;
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let key = SymmetricKey::new(vec![0x7f; 3])?;
    let ciphertext = xor_block_cipher(&Plaintext::new(b"abc".to_vec()), &key)?;
    assert_screen_reader_friendly(&encode_hex(&ciphertext), "XOR block output");
    let error = error_text(xor_block_cipher(&Plaintext::new(b"ab".to_vec()), &key));
    assert_screen_reader_friendly(&error, "an XOR block size error");
    Ok(())
}

#[test]
fn test_xor_stream_output_and_errors_are_plain_ascii() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::{xor_stream_cipher, xor_stream_decipher};
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let key = SymmetricKey::new(vec![0x7f; 3])?;
    let ciphertext = xor_stream_cipher(&Plaintext::new(b"abcdefg".to_vec()), &key)?;
    assert_screen_reader_friendly(&encode_hex(&ciphertext), "XOR stream output");
    let error = error_text(xor_stream_decipher(&[0; 4], &key));
    assert_screen_reader_friendly(&error, "an XOR stream length error");
    Ok(())
}

#[test]
fn test_invalid_aes_key_message_is_plain_ascii() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    use my_pgp_lib::SymmetricKey;
    let message = error_text(expand_key(&SymmetricKey::new(vec![0; 5])?));
    assert_screen_reader_friendly(&message, "an AES key size error");
    Ok(())
}
