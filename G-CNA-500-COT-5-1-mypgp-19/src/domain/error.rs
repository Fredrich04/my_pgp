use std::error::Error;
use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, MyPgpError>;

#[derive(Debug)]
pub enum MyPgpError {
    InvalidArguments(&'static str),
    InvalidHexLength {
        digits: usize,
    },
    InvalidHexDigit {
        position: usize,
    },
    InvalidKeyLength {
        bytes: usize,
    },
    BlockSizeMismatch {
        message_bytes: usize,
        key_bytes: usize,
    },
    CiphertextLengthNotMultiple {
        ciphertext_bytes: usize,
        key_bytes: usize,
    },
    InputTooLarge {
        limit_bytes: usize,
    },
    NumberTooLarge {
        max_bytes: usize,
    },
    Io(io::Error),
    NotImplemented(&'static str),
}

impl fmt::Display for MyPgpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArguments(reason) => write!(f, "invalid arguments: {}", reason),
            Self::InvalidHexLength { digits } => {
                write!(f, "invalid hexadecimal: odd length ({} digits)", digits)
            }
            Self::InvalidHexDigit { position } => {
                write!(f, "invalid hexadecimal: bad digit at position {}", position)
            }
            Self::InvalidKeyLength { bytes } => write!(f, "invalid key length: {} bytes", bytes),
            Self::BlockSizeMismatch { message_bytes, key_bytes } => write!(
                f,
                "block mode requires message and key of the same size (message: {} bytes, key: {} bytes)",
                message_bytes, key_bytes
            ),
            Self::CiphertextLengthNotMultiple {
                ciphertext_bytes,
                key_bytes,
            } => write!(
                f,
                "ciphertext length ({} bytes) is not a multiple of the key size ({} bytes)",
                ciphertext_bytes, key_bytes
            ),
            Self::InputTooLarge { limit_bytes } => {
                write!(f, "input exceeds the maximum size of {} bytes", limit_bytes)
            }
            Self::NumberTooLarge { max_bytes } => {
                write!(f, "number does not fit in {} bytes", max_bytes)
            }
            Self::Io(source) => write!(f, "I/O error: {}", source),
            Self::NotImplemented(feature) => write!(f, "not implemented: {}", feature),
        }
    }
}

impl Error for MyPgpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::InvalidArguments(_)
            | Self::InvalidHexLength { .. }
            | Self::InvalidHexDigit { .. }
            | Self::InvalidKeyLength { .. }
            | Self::BlockSizeMismatch { .. }
            | Self::CiphertextLengthNotMultiple { .. }
            | Self::InputTooLarge { .. }
            | Self::NumberTooLarge { .. }
            | Self::NotImplemented(_) => None,
        }
    }
}

impl From<io::Error> for MyPgpError {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}
