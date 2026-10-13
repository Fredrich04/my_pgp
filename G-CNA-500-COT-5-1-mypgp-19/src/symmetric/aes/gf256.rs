const REDUCTION: u8 = 0x1b;

pub const fn gf_mul(left: u8, right: u8) -> u8 {
    let mut a = left;
    let mut b = right;
    let mut product = 0u8;
    while b != 0 {
        if b & 1 != 0 {
            product ^= a;
        }
        let carry = a & 0x80 != 0;
        a <<= 1;
        if carry {
            a ^= REDUCTION;
        }
        b >>= 1;
    }
    product
}

pub const fn gf_inverse(value: u8) -> u8 {
    let mut result = 1u8;
    let mut base = value;
    let mut exponent = 254u8;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = gf_mul(result, base);
        }
        base = gf_mul(base, base);
        exponent >>= 1;
    }
    result
}
