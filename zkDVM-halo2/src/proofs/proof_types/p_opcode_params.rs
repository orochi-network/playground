use super::p_numeric_encoding::PNumericEncoding;

pub struct POpcodeParam {
    value: u32,
}

impl PNumericEncoding for POpcodeParam {
    fn to_u32(&self) -> u32 {
        self.value
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: v
        }
    }
}