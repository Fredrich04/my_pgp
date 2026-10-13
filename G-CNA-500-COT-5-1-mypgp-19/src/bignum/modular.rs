use super::BigUint;

impl BigUint {
    pub fn powmod(&self, exp: &BigUint, m: &BigUint) -> BigUint {
        assert!(!m.is_zero(), "powmod: modulo nul");
        if *m == BigUint::one() {
            return BigUint::zero();
        }
        let mut result = BigUint::one();
        let mut base = self.rem(m);
        for i in 0..exp.bit_length() {
            if exp.get_bit(i) {
                result = result.mul(&base).rem(m);
            }
            base = base.mul(&base).rem(m);
        }
        result
    }

    pub fn gcd(&self, other: &BigUint) -> BigUint {
        let mut a = self.clone();
        let mut b = other.clone();
        while !b.is_zero() {
            let r = a.rem(&b);
            a = b;
            b = r;
        }
        a
    }

    pub fn mod_inverse(&self, m: &BigUint) -> Option<BigUint> {
        if m.is_zero() {
            return None;
        }
        let mut old_r = self.rem(m);
        let mut r = m.clone();
        let mut old_s = BigUint::one();
        let mut s = BigUint::zero();

        while !r.is_zero() {
            let (q, new_r) = old_r.div_mod(&r);
            old_r = std::mem::replace(&mut r, new_r);

            let qs = q.mul(&s).rem(m);
            let new_s = old_s.add(m).sub(&qs).rem(m);
            old_s = std::mem::replace(&mut s, new_s);
        }

        if old_r == BigUint::one() {
            Some(old_s.rem(m))
        } else {
            None
        }
    }
}
