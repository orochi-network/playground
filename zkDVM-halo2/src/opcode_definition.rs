#[derive(Clone)]
pub enum Opcode {
    Add = 0x01, // top is decreased by 1 with written output = lhs + rhs, pc += 1
    Sub = 0x02, // top is decreased by 1 with written output = lhs - rhs, pc += 1
    Mul = 0x03, // top is decreased by 1 with written output = lhs * rhs, pc += 1
    Div = 0x04, // top is decreased by 1 with written output = lhs // rhs, if rhs == 0 then jump to ErrDivisionByZero, pc += 1
    Push = 0x05, // top is increased by 1, no constraint to lhs and rhs, pc += 1
    Pop = 0x06, // top is decreased by 1, no constraint to lhs and rhs, pc += 1
    Ret = 0x07, // top is kept unchanged with written output = rhs, pc unchanged
    Swap = 0x08, // top is kept unchanged with written stack[top - 1] and stackp[top] swapped, pc += 1
    Jump = 0x09, // top is kept unchanged with written output = rhs, pc = new_pc
    Jumpi = 0x0a, // top is kept unchanged with written output = rhs, pc = (bool(rhs)) * (pc + 1) + (1 - bool(rhs)) * new_pc
    Err = 0xfe, // top is increased by 1 with written error code (1 param), pc unchanged
}

pub enum ErrorCode {
    NoError = 0x00, // there is no error happened
    DivisionByZero = 0x01, // divison by zero
    IncorrectStackAccess = 0x02, // incorrect stack access 
}

pub trait NumericEncoding {
    fn to_u32(&self) -> u32;
    fn from_u32(v: u32) -> Self;
}

pub trait StackTopRequirement {
    fn get_stack_top_minimum(&self) -> usize;
}

impl NumericEncoding for Opcode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        let res = match v {
            0 => Self::Push,
            0x01 => Self::Add,
            0x02 => Self::Sub,
            0x03 => Self::Mul,
            0x04 => Self::Div,
            0x07 => Self::Ret,
            0x08 => Self::Swap,
            0x09 => Self::Jump,
            0x0a => Self::Jumpi,
            0xfe => Self::Err,
            _ => Self::Err,
        };
        assert_eq!(res.to_u32(), v);
        res
    }
}

impl NumericEncoding for ErrorCode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        assert!(0 == 1);
        Self::NoError
    }
}

impl StackTopRequirement for Opcode {
    fn get_stack_top_minimum(&self) -> usize {
        match self {
            Opcode::Add => 3, // 2 stack params
            Opcode::Sub => 3, // 2 stack params
            Opcode::Mul => 3, // 2 stack params
            Opcode::Div => 3, // 2 stack params
            Opcode::Push => 1, // 1 external param required, no stack param
            Opcode::Pop => 2, // 1 stack param
            Opcode::Ret => 2, // 1 stack param
            Opcode::Swap => 3, // 2 stack params
            Opcode::Jump => 2, // 1 stack param
            Opcode::Jumpi => 2, // 2 stack params
            Opcode::Err => 2, // 1 stack param
        }
    }
}

pub struct OpcodeWithParams {
    opcode: Opcode,
    param: Option<u32>,
}

impl OpcodeWithParams {
    pub fn new(opcode: Opcode, param: Option<u32>) -> Self {
        Self {
            opcode: opcode,
            param: param,
        }
    }

    pub fn get_opcode(&self) -> Opcode {
        self.opcode
    }

    pub fn get_param(&self) -> Option<u32> {
        self.param
    }
}