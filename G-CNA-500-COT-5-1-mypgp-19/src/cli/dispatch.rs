use crate::bignum::BigUint;
use crate::cli::args::{Args, OperationMode};
use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{CryptoSystem, Mode, Plaintext, SymmetricKey};
use crate::io::hex::{decode_hex, encode_hex};
use crate::io::stdin::{read_message, MAX_INPUT_BYTES};
use crate::rsa::{keygen, parse_rsa_key};
use crate::symmetric::aes::block::{decrypt_block, encrypt_block};
use crate::symmetric::aes::stream::{aes_stream_cipher, aes_stream_decipher};
use crate::symmetric::xor::{
    xor_block_cipher, xor_block_decipher, xor_stream_cipher, xor_stream_decipher,
};
use std::io::{self, Write};

type CipherFn = fn(&Plaintext, &SymmetricKey) -> Result<Vec<u8>>;
type DecipherFn = fn(&[u8], &SymmetricKey) -> Result<Plaintext>;

pub struct RsaKey {
    pub exp: BigUint,
    pub n: BigUint,
}

pub enum Operation {
    Cipher {
        key: SymmetricKey,
        transform: CipherFn,
    },
    Decipher {
        key: SymmetricKey,
        transform: DecipherFn,
    },
    RsaGenerate {
        p: BigUint,
        q: BigUint,
    },
    RsaCipher {
        key: RsaKey,
        encipher: bool,
    },
    PgpEncrypt {
        sym_key: SymmetricKey,
        rsa_pub_key: RsaKey,
        transform: CipherFn,
    },
    PgpDecrypt {
        enc_sym_key: Vec<u8>,
        rsa_priv_key: RsaKey,
        transform: DecipherFn,
        is_block: bool,
        is_aes: bool,
    },
}

pub fn run(args: &Args) -> Result<()> {
    execute(decide(args)?)
}

fn decide(args: &Args) -> Result<Operation> {
    match args.system {
        CryptoSystem::Xor => decide_xor(args),
        CryptoSystem::Aes => decide_aes(args),
        CryptoSystem::Rsa => decide_rsa(args),
        CryptoSystem::PgpXor => decide_pgp(args, "xor"),
        CryptoSystem::PgpAes => decide_pgp(args, "aes"),
    }
}

fn decide_xor(args: &Args) -> Result<Operation> {
    let key = symmetric_key_from_hex(&args.key)?;
    match (args.mode, args.operation) {
        (Mode::Cipher, OperationMode::Block) => Ok(Operation::Cipher {
            key,
            transform: xor_block_cipher,
        }),
        (Mode::Cipher, OperationMode::Stream) => Ok(Operation::Cipher {
            key,
            transform: xor_stream_cipher,
        }),
        (Mode::Decipher, OperationMode::Block) => Ok(Operation::Decipher {
            key,
            transform: xor_block_decipher,
        }),
        (Mode::Decipher, OperationMode::Stream) => Ok(Operation::Decipher {
            key,
            transform: xor_stream_decipher,
        }),
        (Mode::Generate, _) => Err(MyPgpError::InvalidArguments("xor has no generate mode")),
    }
}

fn decide_aes(args: &Args) -> Result<Operation> {
    let key = symmetric_key_from_hex(&args.key)?;
    ensure_aes_key_len(key.len())?;
    match (args.mode, args.operation) {
        (Mode::Cipher, OperationMode::Block) => Ok(Operation::Cipher {
            key,
            transform: encrypt_block,
        }),
        (Mode::Cipher, OperationMode::Stream) => Ok(Operation::Cipher {
            key,
            transform: aes_stream_cipher,
        }),
        (Mode::Decipher, OperationMode::Block) => Ok(Operation::Decipher {
            key,
            transform: decrypt_block,
        }),
        (Mode::Decipher, OperationMode::Stream) => Ok(Operation::Decipher {
            key,
            transform: aes_stream_decipher,
        }),
        (Mode::Generate, _) => Err(MyPgpError::InvalidArguments("aes has no generate mode")),
    }
}

fn decide_rsa(args: &Args) -> Result<Operation> {
    match args.mode {
        Mode::Generate => {
            let p_hex = args.p.as_deref().unwrap_or("");
            let q_hex = args.q.as_deref().unwrap_or("");
            let p = parse_rsa_prime(p_hex)?;
            let q = parse_rsa_prime(q_hex)?;
            Ok(Operation::RsaGenerate { p, q })
        }
        Mode::Cipher => {
            let (exp, n) = parse_rsa_key(&args.key)?;
            ensure_rsa_key(&exp, &n)?;
            Ok(Operation::RsaCipher {
                key: RsaKey { exp, n },
                encipher: true,
            })
        }
        Mode::Decipher => {
            let (exp, n) = parse_rsa_key(&args.key)?;
            ensure_rsa_key(&exp, &n)?;
            Ok(Operation::RsaCipher {
                key: RsaKey { exp, n },
                encipher: false,
            })
        }
    }
}

fn decide_pgp(args: &Args, sym_algo: &str) -> Result<Operation> {
    let parts: Vec<&str> = args.key.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(MyPgpError::InvalidArguments(
            "PGP key must be SYMMETRIC_KEY:RSA_KEY",
        ));
    }
    match args.mode {
        Mode::Cipher => {
            let sym_key = symmetric_key_from_hex(parts[0])?;
            if sym_algo == "aes" {
                ensure_aes_key_len(sym_key.len())?;
            }
            let (exp, n) = parse_rsa_key(parts[1])?;
            ensure_rsa_key(&exp, &n)?;
            let transform = match (args.operation, sym_algo) {
                (OperationMode::Block, "xor") => xor_block_cipher,
                (OperationMode::Stream, "xor") => xor_stream_cipher,
                (OperationMode::Block, "aes") => encrypt_block,
                (OperationMode::Stream, "aes") => aes_stream_cipher,
                _ => {
                    return Err(MyPgpError::InvalidArguments(
                        "invalid PGP mode/algo combination",
                    ))
                }
            };
            Ok(Operation::PgpEncrypt {
                sym_key,
                rsa_pub_key: RsaKey { exp, n },
                transform,
            })
        }
        Mode::Decipher => {
            let enc_sym_key = decode_hex(parts[0])?;
            if enc_sym_key.is_empty() {
                return Err(MyPgpError::InvalidArguments(
                    "empty encrypted symmetric key",
                ));
            }
            let (exp, n) = parse_rsa_key(parts[1])?;
            ensure_rsa_key(&exp, &n)?;
            let transform = match (args.operation, sym_algo) {
                (OperationMode::Block, "xor") => xor_block_decipher,
                (OperationMode::Stream, "xor") => xor_stream_decipher,
                (OperationMode::Block, "aes") => decrypt_block,
                (OperationMode::Stream, "aes") => aes_stream_decipher,
                _ => {
                    return Err(MyPgpError::InvalidArguments(
                        "invalid PGP mode/algo combination",
                    ))
                }
            };
            Ok(Operation::PgpDecrypt {
                enc_sym_key,
                rsa_priv_key: RsaKey { exp, n },
                transform,
                is_block: matches!(args.operation, OperationMode::Block),
                is_aes: sym_algo == "aes",
            })
        }
        Mode::Generate => Err(MyPgpError::InvalidArguments("PGP has no generate mode")),
    }
}

pub fn execute(operation: Operation) -> Result<()> {
    match operation {
        Operation::Cipher { key, transform } => {
            let message = read_raw_stdin()?;
            let ciphertext = transform(&message, &key)?;
            println!("{}", encode_hex(&ciphertext));
        }
        Operation::Decipher { key, transform } => {
            let ciphertext = read_hex_stdin()?;
            let plaintext = transform(&ciphertext, &key)?;
            io::stdout().write_all(plaintext.as_bytes())?;
            println!();
        }
        Operation::RsaGenerate { p, q } => {
            let kp = keygen(&p, &q)
                .map_err(|_| MyPgpError::InvalidArguments("key generation failed"))?;
            println!("{}", kp.display());
        }
        Operation::RsaCipher { key, encipher } => {
            if encipher {
                let message = read_raw_stdin()?;
                let ciphertext = rsa_encrypt(&key, message.as_bytes())?;
                println!("{}", encode_hex(&ciphertext));
            } else {
                let ciphertext = read_hex_stdin()?;
                let plaintext = rsa_decrypt(&key, &ciphertext)?;
                io::stdout().write_all(&plaintext)?;
                println!();
            }
        }
        Operation::PgpEncrypt {
            sym_key,
            rsa_pub_key,
            transform,
        } => {
            let message = read_raw_stdin()?;
            let sym_bytes = sym_key.as_bytes().to_vec();
            let enc_sym_key = rsa_encrypt(&rsa_pub_key, &sym_bytes)?;
            let enc_message = transform(&message, &sym_key)?;
            println!("{}", encode_hex(&enc_sym_key));
            println!("{}", encode_hex(&enc_message));
        }
        Operation::PgpDecrypt {
            enc_sym_key,
            rsa_priv_key,
            transform,
            is_block,
            is_aes,
        } => {
            let ciphertext = read_pgp_ciphertext(&encode_hex(&enc_sym_key))?;
            let dec_sym_bytes = rsa_decrypt(&rsa_priv_key, &enc_sym_key)?;
            let padded = pad_decrypted_sym_key(dec_sym_bytes, ciphertext.len(), is_block, is_aes);
            let sym_key = SymmetricKey::new(padded)?;
            let plaintext = transform(&ciphertext, &sym_key)?;
            io::stdout().write_all(plaintext.as_bytes())?;
            println!();
        }
    }
    Ok(())
}

fn read_raw_stdin() -> Result<Plaintext> {
    read_message(io::stdin().lock(), MAX_INPUT_BYTES)
}

fn read_hex_stdin() -> Result<Vec<u8>> {
    let raw = read_message(io::stdin().lock(), MAX_INPUT_BYTES)?;
    let text = std::str::from_utf8(raw.as_bytes())
        .map_err(|_| MyPgpError::InvalidArguments("hex input is not valid UTF-8"))?;
    decode_hex(text.trim())
}

fn read_pgp_ciphertext(encrypted_key: &str) -> Result<Vec<u8>> {
    let raw = read_message(io::stdin().lock(), MAX_INPUT_BYTES)?;
    let text = std::str::from_utf8(raw.as_bytes())
        .map_err(|_| MyPgpError::InvalidArguments("hex input is not valid UTF-8"))?;
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    match lines.as_slice() {
        [ciphertext] => decode_hex(ciphertext),
        [input_key, ciphertext] if input_key.eq_ignore_ascii_case(encrypted_key) => {
            decode_hex(ciphertext)
        }
        [..] => Err(MyPgpError::InvalidArguments(
            "PGP input must contain the encrypted key and message",
        )),
    }
}

fn parse_rsa_prime(value: &str) -> Result<BigUint> {
    let value = value.trim();
    Ok(BigUint::from_bytes_le(&decode_hex(value)?))
}

fn symmetric_key_from_hex(hex: &str) -> Result<SymmetricKey> {
    SymmetricKey::new(decode_hex(hex)?)
}

fn ensure_aes_key_len(bytes: usize) -> Result<()> {
    match bytes {
        16 | 24 | 32 => Ok(()),
        _ => Err(MyPgpError::InvalidKeyLength { bytes }),
    }
}

fn ensure_rsa_key(exp: &BigUint, n: &BigUint) -> Result<()> {
    if exp.is_zero() || *n <= BigUint::one() {
        return Err(MyPgpError::InvalidArguments("invalid RSA key"));
    }
    Ok(())
}

fn pad_decrypted_sym_key(
    mut bytes: Vec<u8>,
    ciphertext_len: usize,
    is_block: bool,
    is_aes: bool,
) -> Vec<u8> {
    if is_aes {
        for candidate in [16usize, 24, 32] {
            if bytes.len() <= candidate {
                bytes.resize(candidate, 0);
                break;
            }
        }
        return bytes;
    }
    if is_block {
        if bytes.len() < ciphertext_len {
            bytes.resize(ciphertext_len, 0);
        }
        return bytes;
    }
    if ciphertext_len == 0 || bytes.len() >= ciphertext_len {
        return bytes;
    }
    let start = bytes.len().max(1);
    for candidate in start..=ciphertext_len {
        if ciphertext_len.is_multiple_of(candidate) {
            bytes.resize(candidate, 0);
            break;
        }
    }
    bytes
}

fn rsa_encrypt(key: &RsaKey, message: &[u8]) -> Result<Vec<u8>> {
    let m = BigUint::from_bytes_le(message);
    if m >= key.n {
        return Err(MyPgpError::NumberTooLarge {
            max_bytes: key.n.to_bytes_le().len(),
        });
    }
    let c = m.powmod(&key.exp, &key.n);
    Ok(c.to_bytes_le())
}

fn rsa_decrypt(key: &RsaKey, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let c = BigUint::from_bytes_le(ciphertext);
    if c >= key.n {
        return Err(MyPgpError::NumberTooLarge {
            max_bytes: key.n.to_bytes_le().len(),
        });
    }
    let m = c.powmod(&key.exp, &key.n);
    Ok(m.to_bytes_le())
}
