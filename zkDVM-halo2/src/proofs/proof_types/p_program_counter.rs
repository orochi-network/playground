use super::p_numeric_encoding::PNumericEncoding;

pub struct PProgramCounter {
    value: usize,
}

impl PNumericEncoding for PProgramCounter {
    fn to_u32(&self) -> u32 {
        todo!()
    }

    fn from_u32(v: u32) -> Self {
        todo!()
    }
}