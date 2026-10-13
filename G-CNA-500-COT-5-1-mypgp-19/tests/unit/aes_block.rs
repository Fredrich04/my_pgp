use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::symmetric::aes::block::{decrypt_block, encrypt_block};
use my_pgp_lib::{MyPgpError, Plaintext, Result, SymmetricKey};

const SUBJECT_KEY_HEX: &str = "57696e74657220697320636f6d696e67";
const SUBJECT_PLAINTEXT: &str = "All men must die";
const SUBJECT_CIPHERTEXT_HEX: &str = "744ce22c385958348f0df26eceb62eef";

#[test]
fn test_encrypt_block_matches_subject_vector() -> Result<()> {
    let key = SymmetricKey::new(decode_hex(SUBJECT_KEY_HEX)?)?;
    let message = Plaintext::new(SUBJECT_PLAINTEXT.as_bytes().to_vec());
    let ciphertext = encrypt_block(&message, &key)?;
    assert_eq!(
        encode_hex(&ciphertext),
        SUBJECT_CIPHERTEXT_HEX,
        "subject vector: \"{}\" under key \"Winter is coming\" must give {}",
        SUBJECT_PLAINTEXT,
        SUBJECT_CIPHERTEXT_HEX
    );
    Ok(())
}

#[test]
fn test_encrypt_block_rejects_message_not_sixteen_bytes() -> Result<()> {
    let key = SymmetricKey::new(decode_hex(SUBJECT_KEY_HEX)?)?;
    let message = Plaintext::new(b"too short".to_vec());
    let error = encrypt_block(&message, &key)
        .err()
        .ok_or(MyPgpError::InvalidArguments("expected an error"))?;
    assert!(
        matches!(
            error,
            MyPgpError::BlockSizeMismatch {
                message_bytes: 9,
                key_bytes: 16,
            }
        ),
        "a 9-byte message must be rejected as a block size mismatch, got {:?}",
        error
    );
    Ok(())
}

#[test]
fn test_encrypt_block_rejects_invalid_key_length() -> Result<()> {
    let key = SymmetricKey::new(vec![0x42; 15])?;
    let message = Plaintext::new(vec![0; 16]);
    let error = encrypt_block(&message, &key)
        .err()
        .ok_or(MyPgpError::InvalidArguments("expected an error"))?;
    assert!(
        matches!(error, MyPgpError::InvalidKeyLength { bytes: 15 }),
        "a 15-byte key must be rejected as an invalid key length, got {:?}",
        error
    );
    Ok(())
}

#[test]
fn test_decrypt_block_matches_subject_vector() -> Result<()> {
    let key = SymmetricKey::new(decode_hex(SUBJECT_KEY_HEX)?)?;
    let ciphertext = decode_hex(SUBJECT_CIPHERTEXT_HEX)?;
    let plaintext = decrypt_block(&ciphertext, &key)?;
    assert_eq!(
        plaintext.as_bytes(),
        SUBJECT_PLAINTEXT.as_bytes(),
        "subject vector: {} under key \"Winter is coming\" must give \"{}\"",
        SUBJECT_CIPHERTEXT_HEX,
        SUBJECT_PLAINTEXT
    );
    Ok(())
}

#[test]
fn test_encrypt_then_decrypt_restores_the_message() -> Result<()> {
    let key = SymmetricKey::new(decode_hex(SUBJECT_KEY_HEX)?)?;
    let message = Plaintext::new(SUBJECT_PLAINTEXT.as_bytes().to_vec());
    let ciphertext = encrypt_block(&message, &key)?;
    let plaintext = decrypt_block(&ciphertext, &key)?;
    assert_eq!(
        plaintext.as_bytes(),
        SUBJECT_PLAINTEXT.as_bytes(),
        "encrypt_block then decrypt_block must be the identity"
    );
    Ok(())
}

#[test]
fn test_decrypt_block_rejects_ciphertext_not_sixteen_bytes() -> Result<()> {
    let key = SymmetricKey::new(decode_hex(SUBJECT_KEY_HEX)?)?;
    let error = decrypt_block(b"too short", &key)
        .err()
        .ok_or(MyPgpError::InvalidArguments("expected an error"))?;
    assert!(
        matches!(
            error,
            MyPgpError::BlockSizeMismatch {
                message_bytes: 9,
                key_bytes: 16,
            }
        ),
        "a 9-byte ciphertext must be rejected as a block size mismatch, got {:?}",
        error
    );
    Ok(())
}
