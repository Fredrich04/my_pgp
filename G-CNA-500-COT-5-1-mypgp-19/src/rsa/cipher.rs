use crate::bignum::BigUint;
use crate::domain::error::{MyPgpError, Result};
use crate::domain::rsa::RsaCipher;
use crate::io::hex::decode_hex;
use crate::rsa::keygen::KeyPair;

fn hex_to_biguint(s: &str) -> Result<BigUint> {
    let bytes = decode_hex(s)?;
    Ok(BigUint::from_bytes_le(&bytes))
}

pub fn parse_rsa_key(key_str: &str) -> Result<(BigUint, BigUint)> {
    let parts: Vec<&str> = key_str.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(MyPgpError::InvalidArguments("invalid RSA key format"));
    }
    let exponent = hex_to_biguint(parts[0])?;
    let modulus = hex_to_biguint(parts[1])?;
    Ok((exponent, modulus))
}

pub fn encrypt_message(key_str: &str, message: &[u8]) -> Result<Vec<u8>> {
    let (exp, n) = parse_rsa_key(key_str)?;
    let m = BigUint::from_bytes_le(message);
    if m >= n {
        return Err(MyPgpError::NumberTooLarge {
            max_bytes: n.to_bytes_le().len(),
        });
    }
    let c = m.powmod(&exp, &n);
    Ok(c.to_bytes_le())
}

pub fn decrypt_message(key_str: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let parts: Vec<&str> = key_str.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(MyPgpError::InvalidArguments("invalid RSA key format"));
    }
    let priv_exp = hex_to_biguint(parts[0])?;
    let n = hex_to_biguint(parts[1])?;
    let c = BigUint::from_bytes_le(ciphertext);
    if c >= n {
        return Err(MyPgpError::NumberTooLarge {
            max_bytes: n.to_bytes_le().len(),
        });
    }
    let m = c.powmod(&priv_exp, &n);
    Ok(m.to_bytes_le())
}

impl RsaCipher for KeyPair {
    fn encrypt(&self, key: &str, message: &[u8]) -> Result<Vec<u8>> {
        encrypt_message(key, message)
    }
    fn decrypt(&self, key: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        decrypt_message(key, ciphertext)
    }
}
