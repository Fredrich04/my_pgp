use crate::domain::error::Result;

pub trait RsaCipher {
    fn encrypt(&self, key: &str, message: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, key: &str, ciphertext: &[u8]) -> Result<Vec<u8>>;
}
