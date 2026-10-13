use my_pgp_lib::io::hex::decode_hex;
use my_pgp_lib::symmetric::aes::round::{
    add_round_key, inv_mix_columns, inv_shift_rows, inv_sub_bytes, mix_columns, shift_rows,
    sub_bytes, State,
};
use my_pgp_lib::Result;

const ROUND_ONE_INPUT: &str = "193de3bea0f4e22b9ac68d2ae9f84808";
const AFTER_SUB_BYTES: &str = "d42711aee0bf98f1b8b45de51e415230";
const AFTER_SHIFT_ROWS: &str = "d4bf5d30e0b452aeb84111f11e2798e5";
const AFTER_MIX_COLUMNS: &str = "046681e5e0cb199a48f8d37a2806264c";
const ROUND_ONE_KEY: &str = "a0fafe1788542cb123a339392a6c7605";
const ROUND_TWO_INPUT: &str = "a49c7ff2689f352b6b5bea43026a5049";

fn state(hex: &str) -> Result<State> {
    let bytes = decode_hex(hex)?;
    let mut state = [0u8; 16];
    assert_eq!(
        bytes.len(),
        16,
        "a test state holds 16 bytes, got {}",
        bytes.len()
    );
    state.copy_from_slice(&bytes);
    Ok(state)
}

fn assert_step(transform: fn(&mut State), input: &str, expected: &str, step: &str) -> Result<()> {
    let mut actual = state(input)?;
    transform(&mut actual);
    assert_eq!(
        actual,
        state(expected)?,
        "FIPS-197 appendix B round 1: {}",
        step
    );
    Ok(())
}

fn assert_undoes(forward: fn(&mut State), inverse: fn(&mut State), name: &str) {
    let original: State = core::array::from_fn(|index| (index as u8).wrapping_mul(37) ^ 0x5a);
    let mut actual = original;
    forward(&mut actual);
    assert_ne!(actual, original, "{} must change a generic state", name);
    inverse(&mut actual);
    assert_eq!(
        actual, original,
        "the inverse of {} must restore the state",
        name
    );
}

#[test]
fn test_sub_bytes_matches_fips_197_appendix_b() -> Result<()> {
    assert_step(sub_bytes, ROUND_ONE_INPUT, AFTER_SUB_BYTES, "SubBytes")
}

#[test]
fn test_shift_rows_matches_fips_197_appendix_b() -> Result<()> {
    assert_step(shift_rows, AFTER_SUB_BYTES, AFTER_SHIFT_ROWS, "ShiftRows")
}

#[test]
fn test_mix_columns_matches_fips_197_appendix_b() -> Result<()> {
    assert_step(
        mix_columns,
        AFTER_SHIFT_ROWS,
        AFTER_MIX_COLUMNS,
        "MixColumns",
    )
}

#[test]
fn test_add_round_key_matches_fips_197_appendix_b() -> Result<()> {
    let mut actual = state(AFTER_MIX_COLUMNS)?;
    add_round_key(&mut actual, &state(ROUND_ONE_KEY)?);
    assert_eq!(
        actual,
        state(ROUND_TWO_INPUT)?,
        "FIPS-197 appendix B round 1: AddRoundKey"
    );
    Ok(())
}

#[test]
fn test_inverse_steps_match_fips_197_appendix_b_backwards() -> Result<()> {
    assert_step(
        inv_mix_columns,
        AFTER_MIX_COLUMNS,
        AFTER_SHIFT_ROWS,
        "InvMixColumns",
    )?;
    assert_step(
        inv_shift_rows,
        AFTER_SHIFT_ROWS,
        AFTER_SUB_BYTES,
        "InvShiftRows",
    )?;
    assert_step(
        inv_sub_bytes,
        AFTER_SUB_BYTES,
        ROUND_ONE_INPUT,
        "InvSubBytes",
    )
}

#[test]
fn test_mix_columns_matches_known_columns() -> Result<()> {
    for (input, expected) in [
        ("db135345", "8e4da1bc"),
        ("f20a225c", "9fdc589d"),
        ("01010101", "01010101"),
        ("c6c6c6c6", "c6c6c6c6"),
        ("d4d4d4d5", "d5d5d7d6"),
        ("2d26314c", "4d7ebdf8"),
    ] {
        let mut actual = state(&input.repeat(4))?;
        mix_columns(&mut actual);
        assert_eq!(
            actual,
            state(&expected.repeat(4))?,
            "MixColumns of column {}",
            input
        );
    }
    Ok(())
}

#[test]
fn test_shift_rows_keeps_the_first_row() -> Result<()> {
    let mut actual = state(ROUND_ONE_INPUT)?;
    shift_rows(&mut actual);
    let original = state(ROUND_ONE_INPUT)?;
    for column in 0..4 {
        assert_eq!(
            actual[4 * column],
            original[4 * column],
            "row 0 is not shifted (column {})",
            column
        );
    }
    Ok(())
}

#[test]
fn test_every_inverse_restores_the_state() {
    assert_undoes(sub_bytes, inv_sub_bytes, "SubBytes");
    assert_undoes(shift_rows, inv_shift_rows, "ShiftRows");
    assert_undoes(mix_columns, inv_mix_columns, "MixColumns");
}

#[test]
fn test_add_round_key_twice_restores_the_state() -> Result<()> {
    let key = state(ROUND_ONE_KEY)?;
    let mut actual = state(ROUND_ONE_INPUT)?;
    add_round_key(&mut actual, &key);
    add_round_key(&mut actual, &key);
    assert_eq!(
        actual,
        state(ROUND_ONE_INPUT)?,
        "AddRoundKey is its own inverse"
    );
    Ok(())
}
