use crc::{Crc, CRC_32_ISO_HDLC};

use crate::chunk_type::ChunkType;

use crate::{Error, Result};

pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        Self { chunk_type, data }
    }

    fn length(&self) -> u32 {
        // The length is the number of bytes in the data field.
        self.data.len() as u32
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn crc(&self) -> u32 {
        Chunk::compute_crc(&self.chunk_type, &self.data)
    }

    /// Returns the data stored in this chunk as a `String`. This function will return an error
    /// if the stored data is not valid UTF-8.
    fn data_as_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.data.clone())?)
    }

    fn as_bytes(&self) -> Vec<u8> {
        let res = self
            .length()
            .to_be_bytes()
            .iter()
            .cloned()
            .chain(self.chunk_type.bytes())
            .chain(self.data.iter().cloned())
            .chain(self.crc().to_be_bytes().iter().cloned())
            .collect::<Vec<u8>>();
        res
    }
}

impl Chunk {
    fn compute_crc(chunk_type: &ChunkType, data: &Vec<u8>) -> u32 {
        let d: Vec<u8> = chunk_type
            .bytes()
            .iter()
            .cloned()
            .chain(data.iter().cloned())
            .collect();

        Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&d[..])
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    /// Parses the first 4 bytes as the length of the supplied data, the next
    /// 4 bytes as the chunk type. The next bytes up until the last 4 bytes to the end are parsed as
    /// the data, the last 4 bytes will be parsed as the crc.
    /// Fails if specified length and actual data's length or the provided and computed crc don't match.
    fn try_from(value: &[u8]) -> Result<Self> {
        // First 4 bytes (one u32) is the length.
        let b_len: [u8; 4] = value[..4]
            .to_vec()
            .try_into()
            .expect("Failed to parse length");
        let be_len = u32::from_be_bytes(b_len);
        let le_len = u32::from_le_bytes(b_len);

        // Next 4 byes is the type.
        let b_type: [u8; 4] = value[4..8]
            .to_vec()
            .try_into()
            .expect("Failed to parse chunk type code");
        let chunk_type = ChunkType::try_from(b_type)?;

        let offset = value.len() - 4;

        // Next n bytes is the data.
        let data: Vec<u8> = value[8..offset].to_vec();

        // Last 4 bytes (one u32) is the crc.
        let b_crc: [u8; 4] = value[offset..]
            .to_vec()
            .try_into()
            .expect("Failed to parse crc");
        let be_crc = u32::from_be_bytes(b_crc);
        let le_crc = u32::from_le_bytes(b_crc);

        let len = data.len() as u32;
        if !(len == be_len || len == le_len) {
            return Err("Data does not have the specified length".into());
        }

        let crc = Chunk::compute_crc(&chunk_type, &data);
        if !(crc == be_crc || crc == le_crc) {
            return Err("Data does not match provided crc".into());
        }

        Ok(Self { chunk_type, data })
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Chunk {{")?;
        writeln!(f, "    Length: {}", self.length())?;
        writeln!(
            f,
            "    Type code: \"{}\" ({:?})",
            self.chunk_type(),
            self.chunk_type.bytes()
        )?;
        writeln!(f, "    Data: {} bytes", self.data.len())?;
        writeln!(f, "    CRC: {}", self.crc())?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn valid_chunk_to_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = testing_chunk();

        assert_eq!(chunk_data, chunk.as_bytes());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
