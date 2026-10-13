use my_pgp_lib::io::hex::{decode_hex, encode_hex};
use my_pgp_lib::symmetric::aes::key_schedule::{expand_key, RoundKeys};
use my_pgp_lib::{MyPgpError, Result, SymmetricKey};

fn expand(hex: &str) -> Result<RoundKeys> {
    expand_key(&SymmetricKey::new(decode_hex(hex)?)?)
}

fn round_key_hex(keys: &RoundKeys, round: usize) -> String {
    keys.round_key(round).map(encode_hex).unwrap_or_default()
}

#[test]
fn test_aes_128_schedule_matches_fips_197_appendix_a1() -> Result<()> {
    let keys = expand("2b7e151628aed2a6abf7158809cf4f3c")?;
    assert_eq!(keys.rounds(), 10, "AES-128 uses 10 rounds (FIPS-197 § 5)");
    assert_eq!(
        round_key_hex(&keys, 0),
        "2b7e151628aed2a6abf7158809cf4f3c",
        "round 0 is the key itself"
    );
    assert_eq!(
        round_key_hex(&keys, 1),
        "a0fafe1788542cb123a339392a6c7605",
        "FIPS-197 A.1: w[4..8]"
    );
    assert_eq!(
        round_key_hex(&keys, 10),
        "d014f9a8c9ee2589e13f0cc8b6630ca6",
        "FIPS-197 A.1: w[40..44]"
    );
    Ok(())
}

#[test]
fn test_aes_192_schedule_matches_fips_197_appendix_a2() -> Result<()> {
    let keys = expand("8e73b0f7da0e6452c810f32b809079e562f8ead2522c6b7b")?;
    assert_eq!(keys.rounds(), 12, "AES-192 uses 12 rounds (FIPS-197 § 5)");
    assert_eq!(
        round_key_hex(&keys, 12),
        "e98ba06f448c773c8ecc720401002202",
        "FIPS-197 A.2: w[48..52]"
    );
    Ok(())
}

#[test]
fn test_aes_256_schedule_matches_fips_197_appendix_a3() -> Result<()> {
    let keys = expand("603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4")?;
    assert_eq!(keys.rounds(), 14, "AES-256 uses 14 rounds (FIPS-197 § 5)");
    assert_eq!(
        round_key_hex(&keys, 2),
        "9ba354118e6925afa51a8b5f2067fcde",
        "FIPS-197 A.3: w[8..12], first SubWord-only step"
    );
    assert_eq!(
        round_key_hex(&keys, 14),
        "fe4890d1e6188d0b046df344706c631e",
        "FIPS-197 A.3: w[56..60]"
    );
    Ok(())
}

#[test]
fn test_round_key_beyond_last_round_is_none() -> Result<()> {
    let keys = expand("000102030405060708090a0b0c0d0e0f")?;
    assert!(
        keys.round_key(10).is_some(),
        "AES-128 has round keys 0 to 10"
    );
    assert!(
        keys.round_key(11).is_none(),
        "there is no round key 11 for AES-128"
    );
    Ok(())
}

#[test]
fn test_invalid_key_sizes_are_rejected() -> Result<()> {
    for size in [1usize, 8, 15, 17, 20, 23, 25, 31, 33, 64] {
        let result = expand_key(&SymmetricKey::new(vec![0; size])?);
        assert!(
            matches!(result, Err(MyPgpError::InvalidKeyLength { bytes }) if bytes == size),
            "AES accepts only 16, 24 or 32 bytes (subject p. 5, FIPS-197 § 5); {} must be rejected, got {:?}",
            size,
            result
        );
    }
    Ok(())
}
