use crate::{Error, Result};

// The (zero-based indexed) 5th bit switches an ASCII character from lower to upper case.
const ASCI_UPPER: u8 = 0b0010_0000;

#[derive(Debug, Eq, PartialEq)]
pub struct ChunkType([u8; 4]);

/// Four bits of the type code, namely bit 5 (value 32) of each byte, are used
/// to convey chunk properties
impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.0
    }

    pub fn bytes_are_alphanumeric(&self) -> bool {
        for byte in self.0 {
            if !ChunkType::is_valid_byte(byte) {
                return false;
            }
        }

        true
    }

    pub fn is_valid(&self) -> bool {
        self.bytes_are_alphanumeric() && self.is_reserved_bit_valid()
    }

    /// Checks if the first char of the type code is uppercase which signals criticality.
    pub fn is_critical(&self) -> bool {
        // First byte holds the ancillary bit.
        self.0[0] & ASCI_UPPER == 0
    }

    pub fn is_public(&self) -> bool {
        // Second byte holds the private bit.
        self.0[1] & ASCI_UPPER == 0
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        // Third byte holds the reserved bit.
        self.0[2] & ASCI_UPPER == 0
    }

    pub fn is_safe_to_copy(&self) -> bool {
        // Fourth byte holds the safe-to-copy bit.
        self.0[3] & ASCI_UPPER != 0
    }
}

impl ChunkType {
    pub fn is_valid_byte(byte: u8) -> bool {
        (byte >= 65 && byte <= 90) || (byte >= 97 && byte <= 122)
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<Self> {
        Ok(Self(value))
    }
}

impl std::str::FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let b = s.as_bytes();
        let s: [u8; 4] = b.try_into()?;
        let chunk = Self(s);

        // Note that we are only checking if the supplied bytes are in the valid ASCII range, not if the
        // reserved bit is actually valid. This is reflected in the tests.
        match chunk.bytes_are_alphanumeric() {
            true => Ok(chunk),
            false => Err("Invalid type chunk code supplied".into()),
        }
    }
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = std::str::from_utf8(&self.0).expect("Failed to convert to utf-8");
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
