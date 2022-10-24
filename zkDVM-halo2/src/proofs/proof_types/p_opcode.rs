use crate::{dummy_virtual_machine::opcode::Opcode, utils::numeric_encoding::NumericEncoding};

use super::p_numeric_encoding::PNumericEncoding;

pub struct POpcode {
    value: Opcode,
}

impl PNumericEncoding for POpcode {
    fn to_u32(&self) -> u32 {
        self.value.to_u32()
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: Opcode::from_u32(v),
        }
    }
}