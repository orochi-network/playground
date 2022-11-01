use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;

use crate::utils::numeric_encoding::NumericEncoding;

#[derive(Debug, FromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessOperation {
    Write = 0x00,
    Read = 0x01,
}

impl NumericEncoding for AccessOperation {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}