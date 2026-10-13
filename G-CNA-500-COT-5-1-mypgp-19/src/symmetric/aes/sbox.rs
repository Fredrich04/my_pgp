use crate::symmetric::aes::gf256::gf_inverse;

const AFFINE_CONSTANT: u8 = 0x63;

pub const SBOX: [u8; 256] = build_sbox();
pub const INV_SBOX: [u8; 256] = invert(&SBOX);

pub fn sub_byte(value: u8) -> u8 {
    SBOX[usize::from(value)]
}

pub fn inv_sub_byte(value: u8) -> u8 {
    INV_SBOX[usize::from(value)]
}

const fn affine(value: u8) -> u8 {
    value
        ^ value.rotate_left(1)
        ^ value.rotate_left(2)
        ^ value.rotate_left(3)
        ^ value.rotate_left(4)
        ^ AFFINE_CONSTANT
}

const fn build_sbox() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut index = 0;
    while index < 256 {
        table[index] = affine(gf_inverse(index as u8));
        index += 1;
    }
    table
}

const fn invert(table: &[u8; 256]) -> [u8; 256] {
    let mut inverse = [0u8; 256];
    let mut index = 0;
    while index < 256 {
        inverse[table[index] as usize] = index as u8;
        index += 1;
    }
    inverse
}
