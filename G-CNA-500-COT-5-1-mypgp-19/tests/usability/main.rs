use my_pgp_lib::io::hex::decode_hex;
use my_pgp_lib::io::little_endian::le_bytes_to_u128;
use my_pgp_lib::io::stdin::read_message;
use my_pgp_lib::MyPgpError;

fn error_text<T: std::fmt::Debug>(result: Result<T, MyPgpError>) -> String {
    match result {
        Err(error) => error.to_string(),
        Ok(value) => format!("<unexpected success: {:?}>", value),
    }
}

#[test]
fn test_odd_length_message_states_the_problem_and_the_length() {
    let message = error_text(decode_hex("abc"));
    assert_eq!(
        message, "invalid hexadecimal: odd length (3 digits)",
        "the user must learn both the problem and the length they typed"
    );
}

#[test]
fn test_bad_digit_message_locates_the_problem() {
    let message = error_text(decode_hex("00zz"));
    assert_eq!(
        message, "invalid hexadecimal: bad digit at position 2",
        "the user must be told where the bad digit is"
    );
}

#[test]
fn test_messages_start_with_the_invalid_domain() {
    for input in ["x0", "0"] {
        let message = error_text(decode_hex(input));
        assert!(
            message.starts_with("invalid hexadecimal"),
            "hex errors must start with the same words for consistency, got {:?} for {:?}",
            message,
            input
        );
    }
}

#[test]
fn test_oversized_number_message_states_the_limit() {
    assert_eq!(
        error_text(le_bytes_to_u128(&[0u8; 20])),
        "number does not fit in 16 bytes",
        "the user must be told the maximum size"
    );
}

#[test]
fn test_oversized_input_message_states_the_limit() {
    assert_eq!(
        error_text(read_message(std::io::Cursor::new(vec![0u8; 10]), 8)),
        "input exceeds the maximum size of 8 bytes",
        "the user must be told the maximum input size"
    );
}

#[test]
fn test_windows_and_unix_line_endings_behave_the_same() -> my_pgp_lib::Result<()> {
    let unix = read_message(std::io::Cursor::new(b"key\n".to_vec()), 64)?;
    let windows = read_message(std::io::Cursor::new(b"key\r\n".to_vec()), 64)?;
    assert_eq!(
        unix.as_bytes(),
        windows.as_bytes(),
        "a message typed on Windows must behave like one typed on Unix"
    );
    Ok(())
}

#[test]
fn test_xor_size_error_tells_both_sizes() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_block_cipher;
    use my_pgp_lib::{Plaintext, SymmetricKey};
    let key = SymmetricKey::new(vec![0; 4])?;
    assert_eq!(
        error_text(xor_block_cipher(&Plaintext::new(vec![0; 3]), &key)),
        "block mode requires message and key of the same size (message: 3 bytes, key: 4 bytes)",
        "the user must see both sizes to fix the input"
    );
    Ok(())
}

#[test]
fn test_xor_stream_length_error_tells_both_sizes() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::xor::xor_stream_decipher;
    use my_pgp_lib::SymmetricKey;
    let key = SymmetricKey::new(vec![0; 4])?;
    assert_eq!(
        error_text(xor_stream_decipher(&[0; 6], &key)),
        "ciphertext length (6 bytes) is not a multiple of the key size (4 bytes)",
        "the user must see both sizes to understand the truncation"
    );
    Ok(())
}

#[test]
fn test_invalid_aes_key_message_gives_the_size() -> my_pgp_lib::Result<()> {
    use my_pgp_lib::symmetric::aes::key_schedule::expand_key;
    use my_pgp_lib::SymmetricKey;
    assert_eq!(
        error_text(expand_key(&SymmetricKey::new(vec![0; 20])?)),
        "invalid key length: 20 bytes",
        "the user must see the size they gave to compare with 16, 24 or 32"
    );
    Ok(())
}
