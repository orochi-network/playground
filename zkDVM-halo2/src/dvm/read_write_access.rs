use super::numeric_encoding::NumericEncoding;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;

#[derive(Debug, FromPrimitive, Clone, Copy)]
pub enum ReadWriteAccess {
    Write = 0x00,
    Read = 0x01,
}

impl NumericEncoding for ReadWriteAccess {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}