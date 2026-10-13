use super::BigUint;
use std::cmp::Ordering;

impl BigUint {
    pub fn zero() -> Self {
        BigUint { limbs: Vec::new() }
    }

    pub fn one() -> Self {
        Self::from_u64(1)
    }

    pub fn from_u64(v: u64) -> Self {
        BigUint {
            limbs: vec![v as u32, (v >> 32) as u32],
        }
        .normalize()
    }

    pub fn to_u64(&self) -> Option<u64> {
        match self.limbs.len() {
            0 => Some(0),
            1 => Some(self.limbs[0] as u64),
            2 => Some(self.limbs[0] as u64 | ((self.limbs[1] as u64) << 32)),
            _ => None,
        }
    }

    pub fn normalize(mut self) -> Self {
        while let Some(&0) = self.limbs.last() {
            self.limbs.pop();
        }
        self
    }

    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    pub fn is_even(&self) -> bool {
        self.limbs.first().is_none_or(|l| l & 1 == 0)
    }

    pub fn bit_length(&self) -> usize {
        match self.limbs.last() {
            None => 0,
            Some(&top) => (self.limbs.len() - 1) * 32 + (32 - top.leading_zeros() as usize),
        }
    }

    pub fn get_bit(&self, i: usize) -> bool {
        let (l, b) = (i / 32, i % 32);
        l < self.limbs.len() && (self.limbs[l] >> b) & 1 == 1
    }

    pub fn set_bit(&mut self, i: usize) {
        let (l, b) = (i / 32, i % 32);
        if l >= self.limbs.len() {
            self.limbs.resize(l + 1, 0);
        }
        self.limbs[l] |= 1 << b;
    }

    pub fn shl1(&self) -> Self {
        let mut res = Vec::with_capacity(self.limbs.len() + 1);
        let mut carry = 0u32;
        for &l in &self.limbs {
            res.push((l << 1) | carry);
            carry = l >> 31;
        }
        if carry != 0 {
            res.push(carry);
        }
        BigUint { limbs: res }
    }

    pub fn shr1(&self) -> Self {
        let mut res = self.limbs.clone();
        let mut carry = 0u32;
        for l in res.iter_mut().rev() {
            let new = (*l >> 1) | (carry << 31);
            carry = *l & 1;
            *l = new;
        }
        BigUint { limbs: res }.normalize()
    }
}

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.limbs.len().cmp(&other.limbs.len()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        for (a, b) in self.limbs.iter().rev().zip(other.limbs.iter().rev()) {
            match a.cmp(b) {
                Ordering::Equal => {}
                ord => return ord,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
