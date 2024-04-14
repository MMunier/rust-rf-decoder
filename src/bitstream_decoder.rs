pub enum BitOrder {
    LittleEndian,
    BigEndian,
}

pub struct BitStreamDecoder {
    bit_order: BitOrder,
}

impl BitStreamDecoder {
    pub const LE: Self = Self {
        bit_order: BitOrder::LittleEndian,
    };

    pub const BE: Self = Self {
        bit_order: BitOrder::BigEndian,
    };

    pub fn decode(&self, bitstream: &[bool]) -> Vec<u8> {
        assert!(bitstream.len() & 7 == 0);
        let mut bitstream_iter = bitstream.iter();
        let mut bytestream = Vec::with_capacity(bitstream.len() / 8);
        loop {
            let mut acc = 0;
            let bit = match bitstream_iter.next() {
                Some(val) => val,
                _ => break,
            };

            match self.bit_order {
                BitOrder::BigEndian => {
                    acc |= *bit as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc <<= 1;
                    acc |= *bitstream_iter.next().unwrap() as u8;
                }
                BitOrder::LittleEndian => {
                    acc |= *bit as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                    acc |= *bitstream_iter.next().unwrap() as u8;
                    acc = acc.rotate_right(1);
                }
            }

            bytestream.push(acc);
        }

        bytestream
    }
}

#[cfg(test)]
mod tests {
    use super::{BitOrder, BitStreamDecoder};

    #[test]
    fn test_bytestream_decode() {
        let decoder_le = BitStreamDecoder {
            bit_order: BitOrder::LittleEndian,
        };
        let decoder_be = BitStreamDecoder {
            bit_order: BitOrder::BigEndian,
        };

        let input = &[false, false, false, false, true, true, true, true];
        assert_eq!(decoder_le.decode(input)[0], 0xF0u8);
        assert_eq!(decoder_be.decode(input)[0], 0x0Fu8);
    }
}
