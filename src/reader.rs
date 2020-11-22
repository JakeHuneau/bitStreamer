#![allow(dead_code)]
use std::io::{BufReader, Error, ErrorKind, Read};

pub struct Reader<R: Read> {
    byte: [u8; 1],
    byte_offset: usize,
    reader: BufReader<R>,
}

impl<R: Read> Reader<R> {
    pub fn new(inner_reader: R) -> Reader<R> {
        Reader {
            byte: [0],
            byte_offset: 8,
            reader: BufReader::new(inner_reader),
        }
    }

    fn extract_bit(&mut self, byte: u8) -> bool {
        let front_is_one = byte & 0b1000_0000 != 0;
        self.byte[0] <<= 1; // Pushes the front bit off the buffer
        self.byte_offset += 1;
        front_is_one
    }

    pub fn read_bit(&mut self) -> Result<bool, Error> {
        if self.byte_offset == 8 {
            // Refresh the buffer
            let n = self.reader.read(&mut self.byte)?;
            if n == 0 {
                // Didn't read anything at all
                return Err(Error::new(ErrorKind::UnexpectedEof, "Unexpected EOF"));
            }
            self.byte_offset = 0;
        }
        let bit = self.extract_bit(self.byte[0]);
        Ok(bit)
    }

    pub fn read_bits(&mut self, number_of_bits: usize) -> Result<u128, Error> {
        if number_of_bits > 128 {
            // Make sure we're not writing more than 128 bits
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Tried to read more than 128 bits",
            ));
        }
        let mut output: u128 = 0;
        for _ in 0..number_of_bits {
            // Keep reading from front of buffer and create bufer from that
            output = output << 1;
            if self.read_bit()? {
                output = output | 0b1;
            }
        }
        Ok(output)
    }

    pub fn read_byte(&mut self) -> Result<u8, Error> {
        Ok(self.read_bits(8)? as u8)
    }

    pub fn read_bytes(&mut self, number_of_bytes: usize) -> Result<Vec<u8>, Error> {
        let mut result: Vec<u8> = Vec::new();
        for _ in 0..number_of_bytes {
            let new_byte = self.read_byte()?;
            result.push(new_byte);
        }
        Ok(result)
    }

    pub fn get_ref(&mut self) -> &BufReader<R> {
        &self.reader
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    pub fn read_bit() {
        // 251 = 1111_1011
        // 85 = 0101_0101
        let cursor = Cursor::new(vec![251, 85]);
        let mut reader = Reader::new(cursor);

        // 1111_1011
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());

        // 0101_0101
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
    }

    #[test]
    pub fn read_bits() {
        // 251 = 1111_1011
        // 85 = 0101_0101
        let cursor = Cursor::new(vec![251, 85]);
        let mut reader = Reader::new(cursor);

        assert_eq!(reader.read_bits(8).unwrap(), 251);
        assert_eq!(reader.read_bits(8).unwrap(), 85);
    }

    #[test]
    pub fn read_bits_u16() {
        // 33987 = 1000_0100__1100_0011
        let cursor = Cursor::new(vec![132, 195]);
        let mut reader = Reader::new(cursor);

        assert_eq!(reader.read_bits(16).unwrap(), 33987);
    }

    #[test]
    pub fn read_bits_u64() {
        // 9566613174483237893 = 1000_0100__1100_0011__0110_1111__1111_1111__0000_0000__1010_0101__0011_1100_0000_0101
        let cursor = Cursor::new(vec![132, 195, 111, 255, 0, 165, 60, 5]);
        let mut reader = Reader::new(cursor);

        assert_eq!(reader.read_bits(64).unwrap(), 9566613174483237893);
    }

    #[test]
    pub fn read_byte() {
        let cursor = Cursor::new(vec![251, 85]);
        let mut reader = Reader::new(cursor);

        assert_eq!(reader.read_byte().unwrap(), 251);
        assert_eq!(reader.read_byte().unwrap(), 85);
    }

    #[test]
    pub fn read_bytes() {
        let cursor = Cursor::new(vec![251, 85]);
        let mut reader = Reader::new(cursor);

        assert_eq!(reader.read_bytes(2).unwrap(), vec![251, 85]);
    }
}
