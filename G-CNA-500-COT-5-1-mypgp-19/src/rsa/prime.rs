use super::random::{random_odd_bits, random_witness};
use crate::bignum::BigUint;
use std::io;

const SMALL_PRIMES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

pub fn is_probable_prime(n: &BigUint, rounds: usize) -> io::Result<bool> {
    let one = BigUint::one();
    let two = BigUint::from_u64(2);
    if *n < two {
        return Ok(false);
    }

    for &p in SMALL_PRIMES.iter() {
        let bp = BigUint::from_u64(p);
        if *n == bp {
            return Ok(true);
        }
        if n.rem(&bp).is_zero() {
            return Ok(false);
        }
    }

    let n_1 = n.sub(&one);
    let mut d = n_1.clone();
    let mut s = 0usize;
    while d.is_even() {
        d = d.div_mod(&two).0;
        s += 1;
    }

    'witness: for _ in 0..rounds {
        let a = random_witness(n)?;
        let mut x = a.powmod(&d, n);
        if x == one || x == n_1 {
            continue;
        }
        for _ in 1..s {
            x = x.mul(&x).rem(n);
            if x == n_1 {
                continue 'witness;
            }
        }
        return Ok(false);
    }
    Ok(true)
}

pub fn generate_prime(bits: usize) -> io::Result<BigUint> {
    loop {
        let candidate = random_odd_bits(bits)?;
        if is_probable_prime(&candidate, 40)? {
            return Ok(candidate);
        }
    }
}
