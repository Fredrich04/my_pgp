#![forbid(unsafe_code)]
#![allow(unknown_lints)]
#![allow(clippy::chunks_exact_to_as_chunks)]

pub mod bignum;
pub mod cli;
pub mod domain;
pub mod io;
pub mod rsa;
pub mod symmetric;

pub use bignum::{BigUint, HexError};
pub use domain::{CryptoSystem, Mode, MyPgpError, Plaintext, Result, RsaCipher, SymmetricKey};
pub use rsa::{decrypt_message, encrypt_message, keygen, KeyPair};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
