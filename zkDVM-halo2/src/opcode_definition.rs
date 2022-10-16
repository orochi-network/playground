// Opcodes encoding
// PUSH (encoded 0): top is increased by 1, no constraint to lhs and rhs, pc += 1
// ADD (encoded 1): top is decreased by 1 with written output = lhs + rhs, pc += 1
// SUB (encoded 2): top is decreased by 1 with written output = lhs - rhs, pc += 1
// MUL (encoded 3): top is decreased by 1 with written output = lhs * rhs, pc += 1
// DIV (encoded 4): top is decreased by 1 with written output = lhs // rhs, pc += 1
// RET (encoded 5): top is kept unchanged with written output = rhs, pc += 1
// JMP new_pc (encoded 6): top is kept unchanged with written output = rhs, pc = new_pc according to 
// JMPI dest new_pc (encoded 7): top is kept unchanged with written output = rhs, pc = (bool(rhs)) * (pc + 1) + (1 - bool(rhs)) * new_pc

pub enum Opcode {
    PUSH = 0,
    ADD = 1,
    SUB = 2,
    MUL = 3,
    DIV = 4,
    RET = 5,
    JMP = 6,
    JMPI = 7,
}

pub trait NumericEncoding {
    fn to_u32(&self) -> u32;
    fn from_u32(v: u32) -> Self;
}

impl NumericEncoding for Opcode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        let res = match v {
            0 => Self::PUSH,
            1 => Self::ADD,
            2 => Self::SUB,
            3 => Self::MUL,
            4 => Self::DIV,
            5 => Self::RET,
            6 => Self::JMP,
            _ => Self::JMPI,
        };
        assert_eq!(res.to_u32(), v);
        res
    }
}