use super::BigUint;

#[derive(Debug, PartialEq, Eq)]
pub enum HexError {
    OddLength,
    InvalidDigit(char),
}

impl BigUint {
    pub fn from_bytes_le(bytes: &[u8]) -> Self {
        let limbs = bytes
            .chunks(4)
            .map(|c| {
                let mut buf = [0u8; 4];
                buf[..c.len()].copy_from_slice(c);
                u32::from_le_bytes(buf)
            })
            .collect();
        BigUint { limbs }.normalize()
    }

    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut out: Vec<u8> = self.limbs.iter().flat_map(|l| l.to_le_bytes()).collect();
        while out.len() > 1 && out.last().is_some_and(|b| *b == 0) {
            out.pop();
        }
        if out.is_empty() {
            out.push(0);
        }
        out
    }

    pub fn from_hex_le(s: &str) -> Result<Self, HexError> {
        let chars: Vec<char> = s.trim().chars().collect();
        if !chars.len().is_multiple_of(2) {
            return Err(HexError::OddLength);
        }
        let mut bytes = Vec::with_capacity(chars.len() / 2);
        for pair in chars.chunks(2) {
            let hi = pair[0]
                .to_digit(16)
                .ok_or(HexError::InvalidDigit(pair[0]))?;
            let lo = pair[1]
                .to_digit(16)
                .ok_or(HexError::InvalidDigit(pair[1]))?;
            bytes.push((hi * 16 + lo) as u8);
        }
        Ok(Self::from_bytes_le(&bytes))
    }

    pub fn to_hex_le(&self) -> String {
        self.to_bytes_le()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
