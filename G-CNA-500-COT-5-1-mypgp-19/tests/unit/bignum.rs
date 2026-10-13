use my_pgp_lib::bignum::{BigUint, HexError};
use my_pgp_lib::{MyPgpError, Result};

fn n(v: u64) -> BigUint {
    BigUint::from_u64(v)
}

fn big(v: u128) -> BigUint {
    BigUint::from_bytes_le(&v.to_le_bytes())
}

#[test]
fn test_bignum_normalize_zero() -> Result<()> {
    assert!(n(0).is_zero());
    assert_eq!(n(0), BigUint::zero());
    assert_eq!(n(5).to_u64(), Some(5));
    Ok(())
}

#[test]
fn test_bignum_add_with_carry() -> Result<()> {
    assert_eq!(n(u32::MAX as u64).add(&n(1)).to_u64(), Some(1u64 << 32));
    let a = big(u64::MAX as u128);
    assert_eq!(a.add(&a), big(2 * (u64::MAX as u128)));
    Ok(())
}

#[test]
fn test_bignum_sub_with_borrow() -> Result<()> {
    assert_eq!(n(1u64 << 32).sub(&n(1)).to_u64(), Some(u32::MAX as u64));
    assert_eq!(n(10).sub(&n(10)), BigUint::zero());
    Ok(())
}

#[should_panic]
#[test]
fn test_bignum_sub_negative_panics() {
    n(1).sub(&n(2));
}

#[test]
fn test_bignum_mul() -> Result<()> {
    assert_eq!(n(12345).mul(&n(6789)).to_u64(), Some(12345 * 6789));
    let a = big(u64::MAX as u128);
    assert_eq!(a.mul(&a), big((u64::MAX as u128) * (u64::MAX as u128)));
    assert!(n(7).mul(&BigUint::zero()).is_zero());
    Ok(())
}

#[test]
fn test_bignum_div_mod() -> Result<()> {
    let (q, r) = n(100).div_mod(&n(7));
    assert_eq!((q.to_u64(), r.to_u64()), (Some(14), Some(2)));
    let (q, r) = n(3).div_mod(&n(10));
    assert!(q.is_zero());
    assert_eq!(r.to_u64(), Some(3));
    Ok(())
}

#[test]
fn test_bignum_div_mod_big() -> Result<()> {
    let a = big(0x1234_5678_9abc_def0_1122_3344_5566_7788);
    let b = big(0xdead_beef_cafe);
    let (q, r) = a.div_mod(&b);
    assert!(r < b);
    assert_eq!(q.mul(&b).add(&r), a);
    Ok(())
}

#[test]
fn test_bignum_comparisons() -> Result<()> {
    assert!(n(5) < n(6));
    assert!(big(1u128 << 64) > n(u64::MAX));
    assert_eq!(n(42), n(42));
    Ok(())
}

#[test]
fn test_bignum_powmod() -> Result<()> {
    assert_eq!(n(2).powmod(&n(10), &n(1000)).to_u64(), Some(24));
    assert_eq!(n(5).powmod(&n(0), &n(7)).to_u64(), Some(1));
    Ok(())
}

#[test]
fn test_bignum_gcd_and_inverse() -> Result<()> {
    assert_eq!(n(48).gcd(&n(18)).to_u64(), Some(6));
    assert_eq!(
        n(3).mod_inverse(&n(11))
            .ok_or(MyPgpError::InvalidArguments("no inverse"))?
            .to_u64(),
        Some(4)
    );
    assert_eq!(n(6).mod_inverse(&n(9)), None);
    Ok(())
}

#[test]
fn test_bignum_hex_roundtrip() -> Result<()> {
    let x =
        BigUint::from_hex_le("0102").map_err(|_| MyPgpError::InvalidArguments("invalid hex"))?;
    assert_eq!(x.to_u64(), Some(0x0201));
    assert_eq!(x.to_hex_le(), "0102");
    let y = big(0xdead_beef_0123_4567_89ab);
    assert_eq!(
        BigUint::from_hex_le(&y.to_hex_le())
            .map_err(|_| MyPgpError::InvalidArguments("invalid hex"))?,
        y
    );
    Ok(())
}

#[test]
fn test_bignum_hex_errors() -> Result<()> {
    assert_eq!(BigUint::from_hex_le("abc"), Err(HexError::OddLength));
    assert_eq!(BigUint::from_hex_le("zz"), Err(HexError::InvalidDigit('z')));
    Ok(())
}

#[test]
fn test_mini_rsa() -> Result<()> {
    let modulus = n(61 * 53);
    let phi = n(60 * 52);
    let e = n(17);
    let d = e
        .mod_inverse(&phi)
        .ok_or(MyPgpError::InvalidArguments("no inverse"))?;
    assert_eq!(d.to_u64(), Some(2753));
    let m = n(65);
    let c = m.powmod(&e, &modulus);
    assert_eq!(c.to_u64(), Some(2790));
    assert_eq!(c.powmod(&d, &modulus), m);
    Ok(())
}

#[test]
fn test_large_rsa_roundtrip() -> Result<()> {
    let public = BigUint::from_hex_le("010001")
        .map_err(|_| MyPgpError::InvalidArguments("invalid public key"))?;
    let private = BigUint::from_hex_le(
    "81b08f4eb6dd8a4dd21728e5194dfc4e349829c9991c8b5e44b31e6ceee1e56a11d66ef23389be92ef7a4178470693f509c90b86d4a1e1831056ca0757f3e209",
    )
    .map_err(|_| MyPgpError::InvalidArguments("invalid private key"))?;
    let message = BigUint::from_hex_le("57696e74657220697320636f6d696e67")
        .map_err(|_| MyPgpError::InvalidArguments("invalid message"))?;
    let modulus = BigUint::from_hex_le(
        "c9f91a9ff3bd6d84005b9cc8448296330bd23480f8cf8b36fd4edd0a8cd925de139a0076b962f4d57f50d6f9e64e7c41587784488f923dd60136c763fd602fb3",
    )
    .map_err(|_| MyPgpError::InvalidArguments("invalid modulus"))?;
    let encrypted = message.powmod(&public, &modulus);
    assert_eq!(encrypted.powmod(&private, &modulus), message);
    Ok(())
}

const KNUTH_VECTORS: [(&str, &str, &str, &str); 8] = [
    (
        "800000000000000000000003",
        "200000000000000000000001",
        "3",
        "200000000000000000000000",
    ),
    (
        "80000000fffffffe00000000",
        "80000000ffffffff",
        "ffffffff",
        "7fffffffffffffff",
    ),
    (
        "7fffffff800000000000000000000000",
        "800000000000000000000001",
        "fffffffe",
        "7fffffffffffffff00000002",
    ),
    (
        "8000000000000000fffffffe00000000",
        "8000000000000000ffffffff",
        "ffffffff",
        "7fffffffffffffffffffffff",
    ),
    (
        "ffffffffffffffffffffffffffffffffffffffffffffffff",
        "80000000000000000000000000000001",
        "1ffffffffffffffff",
        "7ffffffffffffffe0000000000000000",
    ),
    (
        "ffffffffffffffffffffffffffffffff",
        "ffffffffffffffff",
        "10000000000000001",
        "0",
    ),
    (
        "100000000000000000000000000000000",
        "10000000000000001",
        "ffffffffffffffff",
        "1",
    ),
    ("ffffffffffffffff", "ffffffff", "100000001", "0"),
];

fn from_be_hex(digits: &str) -> BigUint {
    let padded = if digits.len() % 2 == 1 {
        format!("0{digits}")
    } else {
        digits.to_owned()
    };
    let mut bytes: Vec<u8> = (0..padded.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&padded[i..i + 2], 16).unwrap_or(0))
        .collect();
    bytes.reverse();
    BigUint::from_bytes_le(&bytes)
}

#[test]
fn test_bignum_div_mod_knuth_vectors() {
    for (a, b, q, r) in KNUTH_VECTORS {
        let (quotient, remainder) = from_be_hex(a).div_mod(&from_be_hex(b));
        assert_eq!(quotient, from_be_hex(q), "quotient of {a} / {b}");
        assert_eq!(remainder, from_be_hex(r), "remainder of {a} / {b}");
    }
}
