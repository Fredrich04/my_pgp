use my_pgp_lib::symmetric::xor::{
    xor_block_cipher, xor_block_decipher, xor_stream_cipher, xor_stream_decipher,
};
use my_pgp_lib::{MyPgpError, Plaintext, Result, SymmetricKey};

fn key(bytes: &[u8]) -> Result<SymmetricKey> {
    SymmetricKey::new(bytes.to_vec())
}

#[test]
fn test_cipher_reverses_message_before_xor() -> Result<()> {
    let ciphertext = xor_block_cipher(&Plaintext::new(vec![0x01, 0x02]), &key(&[0x10, 0x20])?)?;
    assert_eq!(
        ciphertext,
        vec![0x02 ^ 0x10, 0x01 ^ 0x20],
        "the last message byte meets the first key byte (subject p. 4 vector)"
    );
    Ok(())
}

#[test]
fn test_decipher_restores_message_order() -> Result<()> {
    let message = xor_block_decipher(&[0x12, 0x21], &key(&[0x10, 0x20])?)?;
    assert_eq!(
        message.as_bytes(),
        [0x01, 0x02],
        "deciphering must undo both the XOR and the reversal"
    );
    Ok(())
}

#[test]
fn test_single_byte_block_roundtrips() -> Result<()> {
    let secret = key(&[0xa5])?;
    let ciphertext = xor_block_cipher(&Plaintext::new(vec![0x5a]), &secret)?;
    assert_eq!(ciphertext, vec![0xff], "0x5a XOR 0xa5 must give 0xff");
    let message = xor_block_decipher(&ciphertext, &secret)?;
    assert_eq!(message.as_bytes(), [0x5a], "a 1-byte block must roundtrip");
    Ok(())
}

#[test]
fn test_zero_key_only_reverses() -> Result<()> {
    let ciphertext = xor_block_cipher(&Plaintext::new(b"abc".to_vec()), &key(&[0, 0, 0])?)?;
    assert_eq!(
        ciphertext, b"cba",
        "XOR with zeros leaves bytes unchanged, only the order flips"
    );
    Ok(())
}

#[test]
fn test_cipher_rejects_shorter_message() -> Result<()> {
    let result = xor_block_cipher(&Plaintext::new(vec![1, 2]), &key(&[1, 2, 3])?);
    assert!(
        matches!(
            result,
            Err(MyPgpError::BlockSizeMismatch {
                message_bytes: 2,
                key_bytes: 3
            })
        ),
        "-b requires message and key of the same size (subject p. 3), got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_cipher_rejects_longer_message() -> Result<()> {
    let result = xor_block_cipher(&Plaintext::new(vec![1, 2, 3, 4]), &key(&[1, 2, 3])?);
    assert!(
        matches!(
            result,
            Err(MyPgpError::BlockSizeMismatch {
                message_bytes: 4,
                key_bytes: 3
            })
        ),
        "a longer message must be rejected in block mode, got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_cipher_rejects_empty_message() -> Result<()> {
    let result = xor_block_cipher(&Plaintext::new(Vec::new()), &key(&[1])?);
    assert!(
        matches!(
            result,
            Err(MyPgpError::BlockSizeMismatch {
                message_bytes: 0,
                key_bytes: 1
            })
        ),
        "an empty message cannot fill a block, got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_decipher_rejects_size_mismatch() -> Result<()> {
    let result = xor_block_decipher(&[1, 2], &key(&[1, 2, 3])?);
    assert!(
        matches!(
            result,
            Err(MyPgpError::BlockSizeMismatch {
                message_bytes: 2,
                key_bytes: 3
            })
        ),
        "a ciphertext of the wrong size must be rejected, got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_stream_single_full_block_equals_block_mode() -> Result<()> {
    let secret = key(&[0x10, 0x20, 0x30])?;
    let message = Plaintext::new(b"abc".to_vec());
    assert_eq!(
        xor_stream_cipher(&message, &secret)?,
        xor_block_cipher(&message, &secret)?,
        "a message of exactly one key size must give the same output in stream and block mode"
    );
    Ok(())
}

#[test]
fn test_stream_cipher_processes_blocks_in_order_and_pads_with_zeros() -> Result<()> {
    let ciphertext = xor_stream_cipher(&Plaintext::new(b"abc".to_vec()), &key(&[0x01, 0x02])?)?;
    assert_eq!(
        ciphertext,
        vec![0x63, 0x63, 0x01, 0x61],
        "blocks 'ab' then 'c\\0' (zero padded) must each go through block mode, in reading order"
    );
    Ok(())
}

#[test]
fn test_stream_cipher_of_empty_message_is_empty() -> Result<()> {
    let ciphertext = xor_stream_cipher(&Plaintext::new(Vec::new()), &key(&[0x01])?)?;
    assert!(
        ciphertext.is_empty(),
        "an empty message has no block to cipher, got {:?}",
        ciphertext
    );
    Ok(())
}

#[test]
fn test_stream_decipher_keeps_padding_zeros() -> Result<()> {
    let message = xor_stream_decipher(&[0x63, 0x63, 0x01, 0x61], &key(&[0x01, 0x02])?)?;
    assert_eq!(
        message.as_bytes(),
        b"abc\0",
        "padding zeros are not removed on decipher (team decision of 2026-09-24)"
    );
    Ok(())
}

#[test]
fn test_stream_decipher_of_empty_input_is_empty() -> Result<()> {
    let message = xor_stream_decipher(&[], &key(&[0x01, 0x02])?)?;
    assert!(
        message.is_empty(),
        "no block means no message, got {:?}",
        message
    );
    Ok(())
}

#[test]
fn test_stream_decipher_rejects_length_not_multiple_of_key() -> Result<()> {
    let result = xor_stream_decipher(&[1, 2, 3], &key(&[1, 2])?);
    assert!(
        matches!(
            result,
            Err(MyPgpError::CiphertextLengthNotMultiple {
                ciphertext_bytes: 3,
                key_bytes: 2
            })
        ),
        "a truncated stream ciphertext must be rejected (team decision of 2026-09-24), got {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_stream_roundtrip_with_exact_multiple() -> Result<()> {
    let secret = key(&[0x5a, 0xa5, 0x3c])?;
    let original = b"Winter is coming".repeat(3);
    let message = xor_stream_decipher(
        &xor_stream_cipher(&Plaintext::new(original.clone()), &secret)?,
        &secret,
    )?;
    assert_eq!(
        message.as_bytes(),
        original.as_slice(),
        "a 48-byte message with a 3-byte key must roundtrip without padding"
    );
    Ok(())
}
