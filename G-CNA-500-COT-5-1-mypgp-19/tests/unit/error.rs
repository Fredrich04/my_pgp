use my_pgp_lib::MyPgpError;

fn assert_display(error: &MyPgpError, expected: &str) {
    assert_eq!(
        error.to_string(),
        expected,
        "{:?} must display a fixed message without secret content (REGLES.md § 4.4)",
        error
    );
}

fn assert_no_source(error: &MyPgpError) {
    assert!(
        std::error::Error::source(error).is_none(),
        "{:?} has no underlying cause and must not report one (REGLES.md § 3.2)",
        error
    );
}

#[test]
fn test_invalid_arguments_display() {
    assert_display(
        &MyPgpError::InvalidArguments("missing key"),
        "invalid arguments: missing key",
    );
}

#[test]
fn test_hex_errors_display_position_not_content() {
    assert_display(
        &MyPgpError::InvalidHexDigit { position: 3 },
        "invalid hexadecimal: bad digit at position 3",
    );
    assert_display(
        &MyPgpError::InvalidHexLength { digits: 5 },
        "invalid hexadecimal: odd length (5 digits)",
    );
}

#[test]
fn test_key_and_block_errors_display_sizes_only() {
    assert_display(
        &MyPgpError::InvalidKeyLength { bytes: 7 },
        "invalid key length: 7 bytes",
    );
    assert_display(
        &MyPgpError::BlockSizeMismatch {
            message_bytes: 10,
            key_bytes: 16,
        },
        "block mode requires message and key of the same size (message: 10 bytes, key: 16 bytes)",
    );
}

#[test]
fn test_input_too_large_display() {
    assert_display(
        &MyPgpError::InputTooLarge { limit_bytes: 1024 },
        "input exceeds the maximum size of 1024 bytes",
    );
}

#[test]
fn test_non_io_errors_have_no_source() {
    for error in [
        MyPgpError::InvalidArguments("x"),
        MyPgpError::InvalidHexLength { digits: 1 },
        MyPgpError::InvalidHexDigit { position: 0 },
        MyPgpError::InvalidKeyLength { bytes: 3 },
        MyPgpError::BlockSizeMismatch {
            message_bytes: 1,
            key_bytes: 2,
        },
        MyPgpError::CiphertextLengthNotMultiple {
            ciphertext_bytes: 5,
            key_bytes: 2,
        },
        MyPgpError::InputTooLarge { limit_bytes: 1 },
        MyPgpError::NumberTooLarge { max_bytes: 16 },
        MyPgpError::NotImplemented("x"),
    ] {
        assert_no_source(&error);
    }
}

#[test]
fn test_not_implemented_display() {
    assert_display(&MyPgpError::NotImplemented("cli"), "not implemented: cli");
}

#[test]
fn test_io_error_keeps_its_source() {
    let io = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "stdin closed");
    let err = MyPgpError::from(io);
    assert!(
        matches!(&err, MyPgpError::Io(inner) if inner.kind() == std::io::ErrorKind::UnexpectedEof),
        "From<io::Error> must keep the original error kind (REGLES.md § 11.5), got {:?}",
        err
    );
    assert!(
        std::error::Error::source(&err).is_some(),
        "an Io error must expose its cause through source() (REGLES.md § 3.2)"
    );
}

#[test]
fn test_number_too_large_display() {
    let err = MyPgpError::NumberTooLarge { max_bytes: 16 };
    assert_display(&err, "number does not fit in 16 bytes");
    assert_no_source(&err);
}

#[test]
fn test_ciphertext_length_not_multiple_display() {
    assert_display(
        &MyPgpError::CiphertextLengthNotMultiple {
            ciphertext_bytes: 5,
            key_bytes: 2,
        },
        "ciphertext length (5 bytes) is not a multiple of the key size (2 bytes)",
    );
}
