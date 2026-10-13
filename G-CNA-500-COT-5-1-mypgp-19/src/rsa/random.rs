use crate::bignum::BigUint;
use std::fs::File;
use std::io::{self, Read};

pub fn random_bytes(n: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    File::open("/dev/urandom")?.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn random_odd_bits(bits: usize) -> io::Result<BigUint> {
    assert!(bits >= 2);
    let nbytes = bits.div_ceil(8);
    let mut b = random_bytes(nbytes)?;
    let last = nbytes - 1;
    let top = (bits - 1) % 8;

    b[last] &= ((1u16 << (top + 1)) - 1) as u8;
    b[last] |= 1 << top;
    b[0] |= 1;

    Ok(BigUint::from_bytes_le(&b))
}

pub fn random_witness(n: &BigUint) -> io::Result<BigUint> {
    let two = BigUint::from_u64(2);
    let range = n.sub(&BigUint::from_u64(3));
    let nbytes = n.bit_length().div_ceil(8) + 8;
    let r = BigUint::from_bytes_le(&random_bytes(nbytes)?);
    Ok(r.rem(&range.add(&BigUint::one())).add(&two))
}
