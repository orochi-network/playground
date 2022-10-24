use super::p_numeric_encoding::PNumericEncoding;

struct PStackDepth {
    value: usize,
}

impl PNumericEncoding for PStackDepth {
    fn to_u32(&self) -> u32 {
        todo!()
    }

    fn from_u32(v: u32) -> Self {
        todo!()
    }
}