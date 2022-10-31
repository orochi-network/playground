use core::fmt;

use super::p_numeric_encoding::PNumericEncoding;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct POpcodeParam {
    value: u32,
}

impl PNumericEncoding for POpcodeParam {
    fn to_u32(&self) -> u32 {
        self.value
    }

    fn from_u32(v: u32) -> Self {
        Self {
            value: v
        }
    }
}

// impl fmt::Display for POpcodeParam {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", format!("{}: {}", std::any::type_name::<Self>(), self.value))
//     }
// }