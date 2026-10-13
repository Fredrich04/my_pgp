use my_pgp_lib::io::little_endian::{le_bytes_to_u128, u128_to_le_bytes};
use my_pgp_lib::{MyPgpError, Result};

fn assert_encodes(value: u128, expected: &[u8], reason: &str) {
    assert_eq!(
        u128_to_le_bytes(value),
        expected,
        "{:#x}: {}",
        value,
        reason
    );
}

fn assert_decodes(bytes: &[u8], expected: u128, reason: &str) -> Result<()> {
    assert_eq!(
        le_bytes_to_u128(bytes)?,
        expected,
        "{:02x?}: {}",
        bytes,
        reason
    );
    Ok(())
}

#[test]
fn test_subject_example_is_reversed() {
    assert_encodes(
        0x0123_4567,
        &[0x67, 0x45, 0x23, 0x01],
        "the subject (p. 4) writes 0x01234567 as 67452301",
    );
}

#[test]
fn test_zero_is_a_single_zero_byte() {
    assert_encodes(0, &[0x00], "zero must still produce one byte");
}

#[test]
fn test_high_zero_bytes_are_dropped_but_inner_zeros_kept() {
    assert_encodes(
        0x0100,
        &[0x00, 0x01],
        "inner zero bytes carry value and must be kept",
    );
    assert_encodes(0xff, &[0xff], "leading zero bytes must be dropped");
}

#[test]
fn test_maximum_value_uses_sixteen_bytes() {
    assert_encodes(u128::MAX, &[0xff; 16], "u128::MAX needs all 16 bytes");
}

#[test]
fn test_decode_subject_example() -> Result<()> {
    assert_decodes(
        &[0x67, 0x45, 0x23, 0x01],
        0x0123_4567,
        "the subject (p. 4) reads 67452301 as 0x01234567",
    )
}

#[test]
fn test_decode_empty_is_zero() -> Result<()> {
    assert_decodes(&[], 0, "no bytes means the number zero")
}

#[test]
fn test_decode_accepts_high_zero_padding() -> Result<()> {
    assert_decodes(
        &[0x05, 0x00, 0x00],
        5,
        "trailing zero bytes are high-order zeros",
    )
}

#[test]
fn test_decode_accepts_exactly_sixteen_bytes() -> Result<()> {
    assert_decodes(&[0xff; 16], u128::MAX, "16 bytes is the u128 capacity")
}

#[test]
fn test_decode_rejects_seventeen_bytes() {
    let result = le_bytes_to_u128(&[0x00; 17]);
    assert!(
        matches!(result, Err(MyPgpError::NumberTooLarge { max_bytes: 16 })),
        "17 bytes exceed a u128 and must be rejected (REGLES.md § 17), got {:?}",
        result
    );
}
