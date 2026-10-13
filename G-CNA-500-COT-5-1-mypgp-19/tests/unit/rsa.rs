use my_pgp_lib::bignum::BigUint;
use my_pgp_lib::rsa::keygen;
use my_pgp_lib::{MyPgpError, Result, RsaCipher};

fn h(s: &str) -> Result<BigUint> {
    BigUint::from_hex_le(s).map_err(|_| MyPgpError::InvalidArguments("invalid hex"))
}

struct ReversingCipher;

impl RsaCipher for ReversingCipher {
    fn encrypt(&self, key: &str, message: &[u8]) -> Result<Vec<u8>> {
        if key.is_empty() {
            return Err(MyPgpError::InvalidArguments("empty rsa key"));
        }
        Ok(message.iter().rev().copied().collect())
    }

    fn decrypt(&self, key: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.encrypt(key, ciphertext)
    }
}

fn roundtrip(cipher: &dyn RsaCipher, key: &str, message: &[u8]) -> Result<Vec<u8>> {
    let encrypted = cipher.encrypt(key, message)?;
    cipher.decrypt(key, &encrypted)
}

#[test]
fn test_trait_is_object_safe_and_roundtrips() -> Result<()> {
    let message = [0x01, 0x02, 0x03];
    assert_eq!(
        roundtrip(&ReversingCipher, "03-d3", &message)?,
        message,
        "decrypt(encrypt(m)) must give m back through a &dyn RsaCipher"
    );
    Ok(())
}

#[test]
fn test_trait_propagates_errors() {
    let result = ReversingCipher.encrypt("", &[0x01]);
    assert!(
        matches!(result, Err(MyPgpError::InvalidArguments(_))),
        "an implementation error must reach the caller unchanged (REGLES.md § 3.2), got {:?}",
        result
    );
}

#[test]
fn test_rsa_keygen_small_primes() -> Result<()> {
    let k =
        keygen(&h("d3")?, &h("e3")?).map_err(|_| MyPgpError::InvalidArguments("key gen failed"))?;
    assert_eq!(k.e.to_hex_le(), "0101");
    assert_eq!(k.n.to_hex_le(), "19bb");
    assert_eq!(k.d.to_hex_le(), "9d5b");
    assert_eq!(k.display(), "public key: 0101-19bb\nprivate key: 9d5b-19bb");
    Ok(())
}

#[test]
fn test_rsa_keygen_256_bits() -> Result<()> {
    let p = h("4b1da73924978f2e9c1f04170e46820d648edbee12ccf4d4462af89b080c86e1")?;
    let q = h("bb3ca1e126f7c8751bd81bc8daa226494efb3d128f72ed9f6cacbe96e14166cb")?;
    let k = keygen(&p, &q).map_err(|_| MyPgpError::InvalidArguments("key gen failed"))?;
    assert_eq!(k.e.to_hex_le(), "010001");
    assert_eq!(k.n.to_hex_le(), "c9f91a9ff3bd6d84005b9cc8448296330bd23480f8cf8b36fd4edd0a8cd925de139a0076b962f4d57f50d6f9e64e7c41587784488f923dd60136c763fd602fb3");
    assert_eq!(k.d.to_hex_le(), "81b08f4eb6dd8a4dd21728e5194dfc4e349829c9991c8b5e44b31e6ceee1e56a11d66ef23389be92ef7a4178470693f509c90b86d4a1e1831056ca0757f3e209");
    Ok(())
}

#[test]
fn test_rsa_roundtrip_small() -> Result<()> {
    let k =
        keygen(&h("d3")?, &h("e3")?).map_err(|_| MyPgpError::InvalidArguments("key gen failed"))?;
    let m = BigUint::from_u64(1234);
    assert_eq!(m.powmod(&k.e, &k.n).powmod(&k.d, &k.n), m);
    Ok(())
}
