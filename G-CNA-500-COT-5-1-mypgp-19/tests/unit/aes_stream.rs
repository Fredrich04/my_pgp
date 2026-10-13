use my_pgp_lib::io::hex::decode_hex;
use my_pgp_lib::symmetric::aes::stream::{aes_stream_cipher, aes_stream_decipher};
use my_pgp_lib::{MyPgpError, Plaintext, Result, SymmetricKey};

const SUBJECT_KEY_HEX: &str = "57696e74657220697320636f6d696e67";
const SUBJECT_PLAINTEXT: &str = "All men must die";
const SUBJECT_CIPHERTEXT_HEX: &str = "744ce22c385958348f0df26eceb62eef";

fn subject_key() -> Result<SymmetricKey> {
    SymmetricKey::new(decode_hex(SUBJECT_KEY_HEX)?)
}

#[test]
fn test_stream_single_full_block_equals_block_mode() -> Result<()> {
    let key = subject_key()?;
    let message = Plaintext::new(SUBJECT_PLAINTEXT.as_bytes().to_vec());
    let ciphertext = aes_stream_cipher(&message, &key)?;
    assert_eq!(
        my_pgp_lib::io::hex::encode_hex(&ciphertext),
        SUBJECT_CIPHERTEXT_HEX,
        "a message of exactly one block must give the same output as encrypt_block"
    );
    Ok(())
}

#[test]
fn test_stream_cipher_pads_the_last_block_with_zeros() -> Result<()> {
    let key = subject_key()?;
    let message = Plaintext::new(b"All men".to_vec());
    let ciphertext = aes_stream_cipher(&message, &key)?;
    assert_eq!(
        ciphertext.len(),
        16,
        "a 7-byte message must still produce one padded 16-byte block"
    );
    Ok(())
}

#[test]
fn test_stream_cipher_of_empty_message_is_empty() -> Result<()> {
    let key = subject_key()?;
    let ciphertext = aes_stream_cipher(&Plaintext::new(Vec::new()), &key)?;
    assert!(
        ciphertext.is_empty(),
        "an empty message has no block to cipher, got {:?}",
        ciphertext
    );
    Ok(())
}

#[test]
fn test_stream_decipher_keeps_padding_zeros() -> Result<()> {
    let key = subject_key()?;
    let ciphertext = decode_hex(SUBJECT_CIPHERTEXT_HEX)?;
    let plaintext = aes_stream_decipher(&ciphertext, &key)?;
    assert_eq!(
        plaintext.as_bytes(),
        SUBJECT_PLAINTEXT.as_bytes(),
        "the subject vector has no padding to observe, but must still roundtrip"
    );
    Ok(())
}

#[test]
fn test_stream_decipher_of_empty_input_is_empty() -> Result<()> {
    let key = subject_key()?;
    let plaintext = aes_stream_decipher(&[], &key)?;
    assert!(
        plaintext.is_empty(),
        "no block means no message, got {:?}",
        plaintext
    );
    Ok(())
}

#[test]
fn test_stream_decipher_rejects_length_not_multiple_of_block_size() -> Result<()> {
    let key = subject_key()?;
    let result = aes_stream_decipher(&[0u8; 17], &key);
    assert!(
        matches!(
            result,
            Err(MyPgpError::CiphertextLengthNotMultiple {
                ciphertext_bytes: 17,
                key_bytes: 16,
            })
        ),
        "a ciphertext that is not a multiple of 16 bytes must be rejected, got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_stream_roundtrip_over_several_blocks() -> Result<()> {
    let key = subject_key()?;
    let original = SUBJECT_PLAINTEXT.repeat(5).into_bytes();
    let message = aes_stream_decipher(
        &aes_stream_cipher(&Plaintext::new(original.clone()), &key)?,
        &key,
    )?;
    assert_eq!(
        message.as_bytes()[..original.len()],
        original[..],
        "a multi-block message must roundtrip through several AES blocks"
    );
    Ok(())
}
