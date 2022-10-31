use crate::{runtime::read_write_access::ReadWriteAccess, utils::numeric_encoding::NumericEncoding};

use super::p_numeric_encoding::PNumericEncoding;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PReadWriteAccess {
    value: ReadWriteAccess,
}

impl PNumericEncoding for PReadWriteAccess {
    fn to_u32(&self) -> u32 {
        self.value.to_u32()
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: ReadWriteAccess::from_u32(v),
        }
    }
}