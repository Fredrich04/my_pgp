use crate::bignum::BigUint;

pub struct KeyPair {
    pub e: BigUint,
    pub d: BigUint,
    pub n: BigUint,
}

const FERMAT_PRIMES: [u64; 5] = [65537, 257, 17, 5, 3];

pub fn carmichael(p: &BigUint, q: &BigUint) -> BigUint {
    let one = BigUint::one();
    let p1 = p.sub(&one);
    let q1 = q.sub(&one);
    let g = p1.gcd(&q1);
    p1.mul(&q1).div_mod(&g).0
}

pub fn keygen(p: &BigUint, q: &BigUint) -> Result<KeyPair, String> {
    let one = BigUint::one();
    if *p <= one || *q <= one {
        return Err("P et Q doivent être des nombres premiers > 1".into());
    }
    if p == q {
        return Err("P et Q doivent être différents".into());
    }

    let n = p.mul(q);
    let lambda = carmichael(p, q);

    for &f in FERMAT_PRIMES.iter() {
        let e = BigUint::from_u64(f);
        if e >= lambda {
            continue;
        }
        if let Some(d) = e.mod_inverse(&lambda) {
            return Ok(KeyPair { e, d, n });
        }
    }
    Err("aucun nombre premier de Fermat ne convient pour ces P et Q".into())
}

impl KeyPair {
    pub fn display(&self) -> String {
        format!(
            "public key: {}-{}\nprivate key: {}-{}",
            self.e.to_hex_le(),
            self.n.to_hex_le(),
            self.d.to_hex_le(),
            self.n.to_hex_le()
        )
    }
}
