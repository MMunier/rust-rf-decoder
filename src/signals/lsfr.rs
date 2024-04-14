use num::PrimInt;
pub struct LSFR<T> {
    poly: T,
    state: T,
}

impl<T> LSFR<T> {
    pub fn new(poly: T, state: T) -> Self {
        Self { poly, state }
    }
}

impl Iterator for LSFR<u8> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let out = (self.state & 1u8) != 0;
        let new_bit = (self.state & self.poly).count_ones() as u8 & 1u8;
        self.state = (new_bit << 7) | (self.state >> 1);
        Some(out)
    }
}

macro_rules! bitstream {
    (@parse 1) => { true  };
    (@parse 0) => { false };
    ($($b:tt)+) => {
        &[
            $(
                bitstream!(@parse $b)
            ),+
        ]
    };
}

#[cfg(test)]
mod tests {
    use super::LSFR;

    #[test]
    fn test_lsfr() {
        let mut lsfr = LSFR::new(0b10101001, 0xFF);
        let expected_stream = bitstream!(1 1 1 1 1 1 1 1 0 1 0 0 1 0 0 0 0 0 0 0 1 1 1 0 1 1 0 0 0 0 0 0 1 0 0 1 1 0 1 0);

        for i in expected_stream {
            assert_eq!(lsfr.next().unwrap(), *i);
        }
    }
}
