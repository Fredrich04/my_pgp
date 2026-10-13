use crate::domain::error::{MyPgpError, Result};
use crate::domain::types::{CryptoSystem, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    Block,
    Stream,
}

pub struct Args {
    pub system: CryptoSystem,
    pub mode: Mode,
    pub operation: OperationMode,
    pub key: String,
    pub p: Option<String>,
    pub q: Option<String>,
}

type ParsedMode = (Mode, OperationMode, String, Option<String>, Option<String>);

const USAGE: &str = "
USAGE\n\
      ./my_pgp CRYPTO_SYSTEM MODE [OPTIONS] [key]\n\
DESCRIPTION\n\
Cipher or decipher MESSAGE using a given CRYPTO_SYSTEM. The MESSAGE is read from the standard input.\n\n\
    CRYPTO_SYSTEM\n\
        \"xor\"             computation using XOR algorithm\n\
        \"aes\"             computation using AES algorithm\n\
        \"rsa\"             computation using RSA algorithm\n\
        \"pgp-xor\"         computation using both RSA and XOR algorithm\n\
        \"pgp-aes\"         computation using both RSA and AES algorithm\n\
    MODE\n\
        -c                  MESSAGE is clear and we want to cipher it\n\
        -d                  MESSAGE is ciphered and we want to decipher it\n\
        -g P Q              RSA only: don't read a MESSAGE, but instead generate a public and private key pair from the prime number P and Q\n\
    OPTIONS\n\
        -b                  for XOR, AES and PGP, only works on one block. The MESSAGE and the symmetric key must be the same size\n\
        key                 Key used to cipher/decipher MESSAGE (incompatible with -g MODE)\n
";

pub fn usage() -> &'static str {
    USAGE
}

pub fn is_help_requested(argv: &[String]) -> bool {
    argv.iter().any(|a| a == "-h" || a == "--help")
}

pub fn parse(argv: &[String]) -> Result<Args> {
    let mut rest = argv.iter().map(String::as_str);
    let system = rest
        .next()
        .ok_or(MyPgpError::InvalidArguments("missing CRYPTO_SYSTEM"))
        .and_then(parse_system)?;
    let mode_str = rest
        .next()
        .ok_or(MyPgpError::InvalidArguments("missing MODE (-c, -d or -g)"))?;
    let (mode, operation, key, p, q) = parse_mode_and_options(mode_str, rest)?;
    Ok(Args {
        system,
        mode,
        operation,
        key,
        p,
        q,
    })
}

fn parse_system(s: &str) -> Result<CryptoSystem> {
    match s {
        "xor" => Ok(CryptoSystem::Xor),
        "aes" => Ok(CryptoSystem::Aes),
        "rsa" => Ok(CryptoSystem::Rsa),
        "pgp-xor" => Ok(CryptoSystem::PgpXor),
        "pgp-aes" => Ok(CryptoSystem::PgpAes),
        _ => Err(MyPgpError::InvalidArguments("unknown CRYPTO_SYSTEM")),
    }
}

fn parse_mode_and_options<'a>(
    mode_str: &str,
    mut rest: impl Iterator<Item = &'a str>,
) -> Result<ParsedMode> {
    let mode = match mode_str {
        "-c" => Mode::Cipher,
        "-d" => Mode::Decipher,
        "-g" => {
            let p = rest
                .next()
                .ok_or(MyPgpError::InvalidArguments("missing P for -g"))?
                .to_owned();
            let q = rest
                .next()
                .ok_or(MyPgpError::InvalidArguments("missing Q for -g"))?
                .to_owned();
            if rest.next().is_some() {
                return Err(MyPgpError::InvalidArguments(
                    "-g takes exactly two arguments",
                ));
            }
            return Ok((
                Mode::Generate,
                OperationMode::Stream,
                String::new(),
                Some(p),
                Some(q),
            ));
        }
        _ => return Err(MyPgpError::InvalidArguments("MODE must be -c, -d or -g")),
    };
    let next = rest
        .next()
        .ok_or(MyPgpError::InvalidArguments("missing key"))?;
    if next == "-b" {
        let key = rest
            .next()
            .ok_or(MyPgpError::InvalidArguments("missing key after -b"))?
            .to_owned();
        if rest.next().is_some() {
            return Err(MyPgpError::InvalidArguments(
                "unexpected argument after key",
            ));
        }
        Ok((mode, OperationMode::Block, key, None, None))
    } else {
        if rest.next().is_some() {
            return Err(MyPgpError::InvalidArguments(
                "unexpected argument after key",
            ));
        }
        Ok((mode, OperationMode::Stream, next.to_owned(), None, None))
    }
}

pub fn validate_args(args: &Args) -> Result<()> {
    match args.mode {
        Mode::Generate => {
            if args.operation != OperationMode::Stream {
                return Err(MyPgpError::InvalidArguments(
                    "-b is incompatible with -g MODE",
                ));
            }
            if args.system != CryptoSystem::Rsa {
                return Err(MyPgpError::InvalidArguments("-g is only valid for RSA"));
            }
        }
        Mode::Cipher | Mode::Decipher => {
            if args.p.is_some() || args.q.is_some() {
                return Err(MyPgpError::InvalidArguments(
                    "P and Q are only valid with -g MODE",
                ));
            }
        }
    }
    Ok(())
}
