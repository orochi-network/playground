use super::p_numeric_encoding::PNumericEncoding;

pub struct PStackValue {
    value: u32,
}

impl PNumericEncoding for PStackValue {
    fn to_u32(&self) -> u32 {
        self.value
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: v,
        }
    }
}