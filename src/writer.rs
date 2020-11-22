#![allow(dead_code)]
use std::io::{BufWriter, Error, ErrorKind, Write};

pub struct Writer<W: Write> {
    byte: [u8; 1],
    byte_offset: usize,
    writer: BufWriter<W>,
}

impl<W: Write> Writer<W> {
    pub fn new(inner_writer: W) -> Writer<W> {
        Writer {
            byte: [0],
            byte_offset: 0,
            writer: BufWriter::new(inner_writer),
        }
    }

    pub fn write_bit(&mut self, write_one: bool) -> Result<(), Error> {
        self.byte[0] <<= 1; // Left shift one so we can add next bit
        if write_one {
            self.byte[0] |= 0b0000_0001;
        }
        self.byte_offset += 1;
        if self.byte_offset == 8 {
            // We're at a full byte, so write it
            let num_bytes_written = self.writer.write(&self.byte)?;
            if num_bytes_written == 0 {
                return Err(Error::new(ErrorKind::WriteZero, "Wrote nothing"));
            }
            self.byte = [0];
            self.byte_offset = 0;
        }
        Ok(())
    }

    pub fn write_bits(&mut self, bits: u128, number_of_bits: usize) -> Result<(), Error> {
        if number_of_bits > 128 {
            // Make sure we're not writing more than 128 bits
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Tried to write more than 128 bits",
            ));
        }

        // Write the bits in order from MSB to LSB by masking everything except the bit we care about
        for mask_location in 1..number_of_bits + 1 {
            let mask: u128 = 1 << (number_of_bits - mask_location);
            self.write_bit(bits & mask != 0)?;
        }
        Ok(())
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        Ok(self.write_bits(byte as u128, 8)?)
    }

    pub fn write_bytes(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        for byte in bytes {
            self.write_byte(byte)?
        }
        Ok(())
    }

    pub fn pad_to_byte(&mut self) -> Result<(), Error> {
        if self.byte_offset != 0 {
            self.write_bits(0, 8 - self.byte_offset)?;
        }
        Ok(())
    }

    pub fn front_pad_to_byte(&mut self) -> Result<(), Error> {
        let num_bytes_written = self.writer.write(&self.byte)?;
        if num_bytes_written == 0 {
            return Err(Error::new(ErrorKind::WriteZero, "Wrote nothing"));
        }
        self.byte = [0];
        self.byte_offset = 0;
        Ok(())
    }

    pub fn get_ref(&mut self) -> &BufWriter<W> {
        &self.writer
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.pad_to_byte()?;
        self.writer.flush()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    pub fn test_write_bit() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        // 1111_1011
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();

        // 0101_0101
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();

        // 101 -> 1010_0000
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();

        writer.flush().unwrap();

        assert_eq!(*writer.get_ref().get_ref().get_ref(), [251, 85, 160]);
    }

    #[test]
    pub fn write_bits() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        writer.write_bits(251, 8).unwrap();
        writer.write_bits(85, 8).unwrap();

        writer.flush().unwrap();

        assert_eq!(*writer.get_ref().get_ref().get_ref(), [251, 85]);
    }

    #[test]
    pub fn write_byte() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        writer.write_byte(251).unwrap();
        writer.write_byte(85).unwrap();

        writer.flush().unwrap();

        assert_eq!(*writer.get_ref().get_ref().get_ref(), [251, 85]);
    }

    #[test]
    pub fn write_bytes() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        writer.write_bytes(vec![251, 85]).unwrap();

        writer.flush().unwrap();

        assert_eq!(*writer.get_ref().get_ref().get_ref(), [251, 85]);
    }

    #[test]
    pub fn write_bytes_u128() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        let test_num: u128 = 340282366920938463463374607431768088333;
        writer.write_bytes(test_num.to_be_bytes().to_vec()).unwrap();

        writer.flush().unwrap();

        assert_eq!(
            *writer.get_ref().get_ref().get_ref(),
            [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 31, 13]
        );
    }

    #[test]
    pub fn pad_to_byte() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        // 1 -> 1000_0000
        writer.write_bit(true).unwrap();
        writer.pad_to_byte().unwrap();

        // 1111_111 -> 1111_1110
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.pad_to_byte().unwrap();

        writer.flush().unwrap();

        assert_eq!(*writer.get_ref().get_ref().get_ref(), [128, 254]);
    }

    #[test]
    pub fn front_pad_to_byte() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = Writer::new(cursor);

        // 1
        writer.write_bit(true).unwrap();
        writer.front_pad_to_byte().unwrap();

        // 101
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.front_pad_to_byte().unwrap();

        // 1010
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.front_pad_to_byte().unwrap();

        writer.flush().unwrap();

        assert_eq!(*writer.get_ref().get_ref().get_ref(), [1, 5, 10]);
    }
}
