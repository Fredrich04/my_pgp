use super::BigUint;

impl BigUint {
    pub fn add(&self, other: &BigUint) -> BigUint {
        let n = self.limbs.len().max(other.limbs.len());
        let mut res = Vec::with_capacity(n + 1);
        let mut carry = 0u64;
        for i in 0..n {
            let a = *self.limbs.get(i).unwrap_or(&0) as u64;
            let b = *other.limbs.get(i).unwrap_or(&0) as u64;
            let sum = a + b + carry;
            res.push(sum as u32);
            carry = sum >> 32;
        }
        if carry != 0 {
            res.push(carry as u32);
        }
        BigUint { limbs: res }.normalize()
    }

    pub fn sub(&self, other: &BigUint) -> BigUint {
        assert!(self >= other, "sub: résultat négatif");
        let mut res = Vec::with_capacity(self.limbs.len());
        let mut borrow = 0i64;
        for i in 0..self.limbs.len() {
            let a = self.limbs[i] as i64;
            let b = *other.limbs.get(i).unwrap_or(&0) as i64;
            let mut diff = a - b - borrow;
            if diff < 0 {
                diff += 1i64 << 32;
                borrow = 1;
            } else {
                borrow = 0;
            }
            res.push(diff as u32);
        }
        BigUint { limbs: res }.normalize()
    }

    pub fn mul(&self, other: &BigUint) -> BigUint {
        if self.is_zero() || other.is_zero() {
            return BigUint::zero();
        }
        let (a, b) = (&self.limbs, &other.limbs);
        let mut res = vec![0u32; a.len() + b.len()];
        for (i, &x) in a.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &y) in b.iter().enumerate() {
                let t = res[i + j] as u64 + (x as u64) * (y as u64) + carry;
                res[i + j] = t as u32;
                carry = t >> 32;
            }
            res[i + b.len()] = carry as u32;
        }
        BigUint { limbs: res }.normalize()
    }
}
