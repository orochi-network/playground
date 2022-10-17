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
    Push = 0, // top is increased by 1, no constraint to lhs and rhs, pc += 1
    Add = 1, // top is decreased by 1 with written output = lhs + rhs, pc += 1
    Sub = 2, // top is decreased by 1 with written output = lhs - rhs, pc += 1
    Mul = 3, // top is decreased by 1 with written output = lhs * rhs, pc += 1
    Div = 4, // top is decreased by 1 with written output = lhs // rhs, if rhs == 0 then jump to ErrDivisionByZero, pc += 1
    Ret = 5, // top is kept unchanged with written output = rhs, pc += 1
    Jmp = 6, // top is kept unchanged with written output = rhs, pc = new_pc
    Jmpi = 7, // top is kept unchanged with written output = rhs, pc = (bool(rhs)) * (pc + 1) + (1 - bool(rhs)) * new_pc
    ErrDivisionByZero = 8, // top is kept unchanged with written output = rhs, pc is unchanged
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
            0 => Self::Push,
            1 => Self::Add,
            2 => Self::Sub,
            3 => Self::Mul,
            4 => Self::Div,
            5 => Self::Ret,
            6 => Self::Jmp,
            7 => Self::Jmpi,
            8 => Self::ErrDivisionByZero,
            _ => Self::Ret,
        };
        assert_eq!(res.to_u32(), v);
        res
    }
}