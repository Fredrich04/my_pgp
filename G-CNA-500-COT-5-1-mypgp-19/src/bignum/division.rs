use super::BigUint;

impl BigUint {
    pub fn div_mod(&self, d: &BigUint) -> (BigUint, BigUint) {
        assert!(!d.is_zero(), "division par zéro");
        if self < d {
            return (BigUint::zero(), self.clone());
        }
        let mut q = BigUint::zero();
        let mut r = BigUint::zero();
        for i in (0..self.bit_length()).rev() {
            r = r.shl1();
            if self.get_bit(i) {
                r.set_bit(0);
            }
            if r >= *d {
                r = r.sub(d);
                q.set_bit(i);
            }
        }
        (q.normalize(), r.normalize())
    }

    pub fn rem(&self, d: &BigUint) -> BigUint {
        self.div_mod(d).1
    }

    pub fn div(&self, d: &BigUint) -> BigUint {
        self.div_mod(d).0
    }
}
