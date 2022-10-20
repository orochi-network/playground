use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::numeric_encoding::NumericEncoding;

// Identify direction for program counter (pc)
#[derive(FromPrimitive, Debug)]
pub enum Direction {
    Normal = 0x00, // normal
    Error = 0x01, // error direction -> force pc to error opcode
    Jump = 0x02, // jump direction -> suddenly change to other pc according to Jump or Jumpi
}

impl NumericEncoding for Direction {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}