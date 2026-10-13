use my_pgp_lib::{CryptoSystem, Mode, MyPgpError, Result, SymmetricKey};

#[test]
fn test_symmetric_key_rejects_empty() {
    let result = SymmetricKey::new(Vec::new());
    assert!(
        matches!(result, Err(MyPgpError::InvalidKeyLength { bytes: 0 })),
        "an empty key must be impossible to build (REGLES.md § 1.3), got {:?}",
        result
    );
}

#[test]
fn test_symmetric_key_exposes_bytes() -> Result<()> {
    let key = SymmetricKey::new(vec![0x57, 0x68, 0x61])?;
    assert_eq!(
        key.as_bytes(),
        &[0x57, 0x68, 0x61],
        "the key must return its bytes unchanged"
    );
    assert_eq!(
        key.len(),
        3,
        "key length is counted in bytes (REGLES.md § 6.3)"
    );
    assert!(!key.is_empty(), "a built key is never empty");
    Ok(())
}

#[test]
fn test_symmetric_key_debug_is_redacted() -> Result<()> {
    let key = SymmetricKey::new(vec![0xde, 0xad, 0xbe, 0xef])?;
    let debug = format!("{:?}", key);
    assert_eq!(
        debug, "SymmetricKey(<4 bytes, redacted>)",
        "Debug must only show the size (REGLES.md § 4.1)"
    );
    assert!(
        !debug.contains("dead") && !debug.contains("239"),
        "no key byte may appear in Debug, in hex or decimal (REGLES.md § 4.1), got {}",
        debug
    );
    Ok(())
}

#[test]
fn test_enum_variants_are_distinct() {
    assert_ne!(
        CryptoSystem::PgpXor,
        CryptoSystem::PgpAes,
        "pgp-xor and pgp-aes are different crypto systems"
    );
    assert_ne!(
        Mode::Cipher,
        Mode::Decipher,
        "-c and -d are different modes"
    );
}

#[test]
fn test_plaintext_exposes_bytes_and_length() {
    let plaintext = my_pgp_lib::Plaintext::new(b"Winter".to_vec());
    assert_eq!(
        plaintext.as_bytes(),
        b"Winter",
        "Plaintext must return its bytes unchanged"
    );
    assert_eq!(
        plaintext.len(),
        6,
        "Plaintext length is counted in bytes (REGLES.md § 6.3)"
    );
    assert!(!plaintext.is_empty(), "a 6-byte plaintext is not empty");
}

#[test]
fn test_plaintext_accepts_empty_message() {
    let plaintext = my_pgp_lib::Plaintext::new(Vec::new());
    assert!(
        plaintext.is_empty(),
        "an empty message is a valid plaintext for stream mode"
    );
}

#[test]
fn test_plaintext_debug_is_redacted() {
    let plaintext = my_pgp_lib::Plaintext::new(b"secret".to_vec());
    let debug = format!("{:?}", plaintext);
    assert_eq!(
        debug, "Plaintext(<6 bytes, redacted>)",
        "Debug must only show the size (REGLES.md § 4.5)"
    );
}
