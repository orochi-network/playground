use crate::{utils::numeric_encoding::NumericEncoding};
use crate::runtime::access_operation_util::access_operation::AccessOperation;

use super::p_numeric_encoding::PNumericEncoding;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PReadWriteAccess {
    value: AccessOperation,
}

impl PNumericEncoding for PReadWriteAccess {
    fn to_u32(&self) -> u32 {
        self.value.to_u32()
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: AccessOperation::from_u32(v),
        }
    }
}