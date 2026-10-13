use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::{MyPgpError, Result};

#[test]
fn test_decode_keeps_byte_order() -> Result<()> {
    assert_eq!(
        decode_hex("67452301")?,
        vec![0x67, 0x45, 0x23, 0x01],
        "decoding must keep the byte order of the text; little-endian only applies to numbers"
    );
    Ok(())
}

#[test]
fn test_decode_accepts_both_cases() -> Result<()> {
    assert_eq!(
        decode_hex("aBcD")?,
        vec![0xab, 0xcd],
        "upper and lower case digits must decode to the same bytes"
    );
    Ok(())
}

#[test]
fn test_decode_empty() -> Result<()> {
    let bytes = decode_hex("")?;
    assert!(
        bytes.is_empty(),
        "empty text must decode to no bytes, got {:?}",
        bytes
    );
    Ok(())
}

#[test]
fn test_decode_rejects_odd_length() {
    let result = decode_hex("abc");
    assert!(
        matches!(result, Err(MyPgpError::InvalidHexLength { digits: 3 })),
        "3 digits cannot form whole bytes (REGLES.md § 17), got {:?}",
        result
    );
}

#[test]
fn test_decode_reports_bad_digit_position() {
    let result = decode_hex("00g1");
    assert!(
        matches!(result, Err(MyPgpError::InvalidHexDigit { position: 2 })),
        "'g' at index 2 must be reported at position 2, got {:?}",
        result
    );
    let result = decode_hex("0 ");
    assert!(
        matches!(result, Err(MyPgpError::InvalidHexDigit { position: 1 })),
        "a space at index 1 must be reported at position 1, got {:?}",
        result
    );
}

#[test]
fn test_decode_rejects_non_ascii() {
    let result = decode_hex("é00");
    assert!(
        matches!(result, Err(MyPgpError::InvalidHexDigit { position: 0 })),
        "a multi-byte character must be rejected at its first byte, got {:?}",
        result
    );
}

#[test]
fn test_encode_empty_gives_empty_string() {
    assert_eq!(encode_hex(&[]), "", "no bytes must give no digits");
}

#[test]
fn test_encode_single_bytes_at_boundaries() {
    for (byte, expected) in [(0x00, "00"), (0x0f, "0f"), (0xf0, "f0"), (0xff, "ff")] {
        assert_eq!(
            encode_hex(&[byte]),
            expected,
            "byte {:#04x} must be written with two digits",
            byte
        );
    }
}

#[test]
fn test_encode_keeps_byte_order() {
    assert_eq!(
        encode_hex(&[0x67, 0x45, 0x23, 0x01]),
        "67452301",
        "encoding must write bytes in their order"
    );
}

#[test]
fn test_encode_is_lowercase() {
    assert_eq!(
        encode_hex(&[0xab, 0xcd, 0xef]),
        "abcdef",
        "the subject prints lowercase hexadecimal"
    );
}

#[test]
fn test_encode_output_has_two_digits_per_byte() {
    let bytes: Vec<u8> = (0..=255).collect();
    assert_eq!(
        encode_hex(&bytes).len(),
        512,
        "256 bytes must produce exactly 512 digits"
    );
}

#[test]
fn test_decode_nominal() -> Result<()> {
    assert_eq!(
        decode_hex("00ff7f80")?,
        vec![0x00, 0xff, 0x7f, 0x80],
        "boundary bytes 00, ff, 7f, 80 must decode exactly"
    );
    Ok(())
}

#[test]
fn test_decode_rejects_single_digit() {
    let result = decode_hex("a");
    assert!(
        matches!(result, Err(MyPgpError::InvalidHexLength { digits: 1 })),
        "a single digit is an odd length, got {:?}",
        result
    );
}

#[test]
fn test_decode_rejects_prefix_0x() {
    let result = decode_hex("0x12");
    assert!(
        matches!(result, Err(MyPgpError::InvalidHexDigit { position: 1 })),
        "the subject never uses a 0x prefix; 'x' must be rejected at position 1, got {:?}",
        result
    );
}
