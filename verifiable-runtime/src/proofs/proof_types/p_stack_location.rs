use super::p_numeric_encoding::PNumericEncoding;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PStackLocation {
    value: usize,
}

impl PNumericEncoding for PStackLocation {
    fn to_u32(&self) -> u32 {
        self.value as u32
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: v as usize,
        }
    }
}