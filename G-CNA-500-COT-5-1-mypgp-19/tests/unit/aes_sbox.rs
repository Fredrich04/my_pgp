use my_pgp_lib::symmetric::aes::gf256::{gf_inverse, gf_mul};
use my_pgp_lib::symmetric::aes::sbox::{inv_sub_byte, sub_byte, INV_SBOX, SBOX};

#[test]
fn test_gf_mul_matches_fips_197_examples() {
    assert_eq!(
        gf_mul(0x57, 0x83),
        0xc1,
        "FIPS-197 § 4.2: {{57}} • {{83}} = {{c1}}"
    );
    assert_eq!(
        gf_mul(0x57, 0x13),
        0xfe,
        "FIPS-197 § 4.2.1: {{57}} • {{13}} = {{fe}}"
    );
}

#[test]
fn test_gf_mul_has_identity_and_zero() {
    for value in 0..=255u8 {
        assert_eq!(
            gf_mul(value, 1),
            value,
            "1 is the neutral element, failed for {:#04x}",
            value
        );
        assert_eq!(
            gf_mul(value, 0),
            0,
            "0 absorbs every element, failed for {:#04x}",
            value
        );
    }
}

#[test]
fn test_gf_mul_is_commutative() {
    for left in (0..=255u8).step_by(7) {
        for right in (0..=255u8).step_by(11) {
            assert_eq!(
                gf_mul(left, right),
                gf_mul(right, left),
                "GF(2^8) multiplication must commute for {:#04x} and {:#04x}",
                left,
                right
            );
        }
    }
}

#[test]
fn test_gf_inverse_times_value_is_one() {
    for value in 1..=255u8 {
        assert_eq!(
            gf_mul(value, gf_inverse(value)),
            1,
            "{:#04x} times its inverse must be 1",
            value
        );
    }
}

#[test]
fn test_gf_inverse_of_zero_is_zero() {
    assert_eq!(
        gf_inverse(0),
        0,
        "FIPS-197 § 5.1.1 maps {{00}} to itself before the affine step"
    );
}

#[test]
fn test_sub_byte_matches_fips_197_values() {
    for (input, expected) in [
        (0x00, 0x63),
        (0x01, 0x7c),
        (0x10, 0xca),
        (0x53, 0xed),
        (0xff, 0x16),
    ] {
        assert_eq!(
            sub_byte(input),
            expected,
            "FIPS-197 Figure 7: S({:#04x})",
            input
        );
    }
}

#[test]
fn test_inv_sub_byte_undoes_sub_byte() {
    for value in 0..=255u8 {
        assert_eq!(
            inv_sub_byte(sub_byte(value)),
            value,
            "InvSubBytes must undo SubBytes for {:#04x}",
            value
        );
    }
}

#[test]
fn test_tables_have_256_entries() {
    assert_eq!(SBOX.len(), 256, "the S-box covers every byte value");
    assert_eq!(
        INV_SBOX.len(),
        256,
        "the inverse S-box covers every byte value"
    );
}
