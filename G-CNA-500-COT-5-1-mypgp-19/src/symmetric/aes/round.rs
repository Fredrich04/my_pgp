use crate::domain::types::wipe;
use crate::symmetric::aes::gf256::gf_mul;
use crate::symmetric::aes::key_schedule::BLOCK_BYTES;
use crate::symmetric::aes::sbox::{inv_sub_byte, sub_byte};

pub type State = [u8; BLOCK_BYTES];

const ROWS: usize = 4;
const COLUMNS: usize = 4;
type Table = [u8; 256];

const TIMES_1: Table = product_table(0x01);
const TIMES_2: Table = product_table(0x02);
const TIMES_3: Table = product_table(0x03);
const TIMES_9: Table = product_table(0x09);
const TIMES_11: Table = product_table(0x0b);
const TIMES_13: Table = product_table(0x0d);
const TIMES_14: Table = product_table(0x0e);
const MIX: [&Table; ROWS] = [&TIMES_2, &TIMES_3, &TIMES_1, &TIMES_1];
const INV_MIX: [&Table; ROWS] = [&TIMES_14, &TIMES_11, &TIMES_13, &TIMES_9];

pub fn sub_bytes(state: &mut State) {
    state.iter_mut().for_each(|byte| *byte = sub_byte(*byte));
}

pub fn inv_sub_bytes(state: &mut State) {
    state
        .iter_mut()
        .for_each(|byte| *byte = inv_sub_byte(*byte));
}

pub fn shift_rows(state: &mut State) {
    rotate_rows(state, |row, column| (column + row) % COLUMNS);
}

pub fn inv_shift_rows(state: &mut State) {
    rotate_rows(state, |row, column| (column + COLUMNS - row) % COLUMNS);
}

pub fn mix_columns(state: &mut State) {
    multiply_columns(state, &MIX);
}

pub fn inv_mix_columns(state: &mut State) {
    multiply_columns(state, &INV_MIX);
}

pub fn add_round_key(state: &mut State, round_key: &[u8]) {
    state
        .iter_mut()
        .zip(round_key)
        .for_each(|(byte, key)| *byte ^= key);
}

fn rotate_rows(state: &mut State, source_column: impl Fn(usize, usize) -> usize) {
    let mut original = *state;
    for column in 0..COLUMNS {
        for row in 0..ROWS {
            state[row + ROWS * column] = original[row + ROWS * source_column(row, column)];
        }
    }
    wipe(&mut original);
}

fn multiply_columns(state: &mut State, coefficients: &[&Table; ROWS]) {
    for column in state.chunks_exact_mut(ROWS) {
        let mut original = [0u8; ROWS];
        original.copy_from_slice(column);
        for (row, output) in column.iter_mut().enumerate() {
            *output = (0..ROWS).fold(0, |sum, index| {
                sum ^ coefficients[(index + ROWS - row) % ROWS][usize::from(original[index])]
            });
        }
        wipe(&mut original);
    }
}

const fn product_table(factor: u8) -> Table {
    let mut table = [0u8; 256];
    let mut value = 0;
    while value < 256 {
        table[value] = gf_mul(factor, value as u8);
        value += 1;
    }
    table
}
