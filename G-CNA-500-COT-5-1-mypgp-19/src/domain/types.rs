use crate::domain::error::{MyPgpError, Result};
use std::fmt;
use std::hint::black_box;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoSystem {
    Xor,
    Aes,
    Rsa,
    PgpXor,
    PgpAes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Cipher,
    Decipher,
    Generate,
}

pub struct SymmetricKey {
    bytes: Vec<u8>,
}

impl SymmetricKey {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        if bytes.is_empty() {
            return Err(MyPgpError::InvalidKeyLength { bytes: 0 });
        }
        Ok(Self { bytes })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for SymmetricKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SymmetricKey(<{} bytes, redacted>)", self.bytes.len())
    }
}

impl Drop for SymmetricKey {
    fn drop(&mut self) {
        wipe(&mut self.bytes);
    }
}

pub struct Plaintext {
    bytes: Vec<u8>,
}

impl Plaintext {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Plaintext(<{} bytes, redacted>)", self.bytes.len())
    }
}

impl Drop for Plaintext {
    fn drop(&mut self) {
        wipe(&mut self.bytes);
    }
}

pub(crate) fn wipe(buffer: &mut [u8]) {
    buffer.fill(0);
    black_box(buffer);
}

#[cfg(test)]
mod tests {
    use super::wipe;

    #[test]
    fn test_wipe_zeroes_buffer() {
        let mut buffer = vec![0xffu8; 32];
        wipe(&mut buffer);
        assert!(
            buffer.iter().all(|&b| b == 0),
            "wipe must zero every byte (REGLES.md § 20.2), got {:?}",
            buffer
        );
    }
}
