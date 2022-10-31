use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum_macros::EnumIter;

use crate::utils::numeric_encoding::NumericEncoding;

#[derive(Clone, FromPrimitive, Debug, PartialEq, Eq, EnumIter)]
pub enum ErrorCode {
    NoError = 0x00, // there is no error happened
    NoReturn = 0x01, // program hasn't stopped
    DivisionByZero = 0x02, // divison by zero
    IncorrectStackAccess = 0x03, // incorrect stack access 
    IncorrectProgramCounter = 0x04, // incorrect program counter
}

impl NumericEncoding for ErrorCode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}