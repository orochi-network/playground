use crate::proofs::proof_types::p_numeric_encoding::PNumericEncoding;
use crate::runtime::opcode_util::opcode::Opcode;
use crate::utils::numeric_encoding::NumericEncoding;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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