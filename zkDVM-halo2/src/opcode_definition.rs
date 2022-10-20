use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;

#[derive(Clone, PartialEq, Eq, FromPrimitive, Debug)]
pub enum Opcode {
    Stop = 0x00, // top is unchanged, program counter is unchanged too
    Add = 0x01, // top is decreased by 1 with written output = lhs + rhs, pc += 1
    Sub = 0x02, // top is decreased by 1 with written output = lhs - rhs, pc += 1
    Mul = 0x03, // top is decreased by 1 with written output = lhs * rhs, pc += 1
    Div = 0x04, // top is decreased by 1 with written output = lhs // rhs, if rhs == 0 then jump to ErrDivisionByZero, pc += 1
    Mod = 0x06, // top is decreased by 1 with writeen output = lhs % rhs, if rhs == 0 then jump to ErrDivisionByZero, pc += 1
    Pop = 0x50, // top is decreased by 1, no constraint to lhs and rhs, pc += 1
    Jump = 0x56, // top is kept unchanged with written output = rhs, pc = new_pc
    Jumpi = 0x57, // top is kept unchanged with written output = rhs, pc = (bool(rhs)) * (pc + 1) + (1 - bool(rhs)) * new_pc
    Push4 = 0x63, // top is increased by 1, no constraint to lhs and rhs, pc += 1
    Dup2 = 0x81, // top is increase by 1 with written output = lhs, pc += 1
    Swap1 = 0x90, // top is kept unchanged with written stack[top - 1] and stackp[top] swapped, pc += 1
    Return = 0xf3, // top is kept unchanged with written output = rhs, pc unchanged
    Error = 0xfe, // top is increased by 1 with written error code (1 param), pc unchanged
}

#[derive(Clone, FromPrimitive, Debug)]
pub enum ErrorCode {
    NoError = 0x00, // there is no error happened
    DivisionByZero = 0x01, // divison by zero
    IncorrectStackAccess = 0x02, // incorrect stack access 
    IncorrectProgramCounter = 0x03, // incorrect program counter
}

pub trait NumericEncoding {
    // transform into u32 value
    fn to_u32(&self) -> u32;

    // from u32 transforming into Self
    fn from_u32(v: u32) -> Self;
}

pub trait StackRequirement {
    // return the minimum depth of the stack to ensure whether the corresponding opcode executed correctly
    fn get_num_stack_params(&self) -> usize;

    // return the number of params needed from the stack for satisfying the opcode's execution
    fn get_stack_depth_minimum(&self) -> usize;
}

impl NumericEncoding for Opcode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}

impl NumericEncoding for ErrorCode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}

impl StackRequirement for Opcode {
    fn get_stack_depth_minimum(&self) -> usize {
        self.get_num_stack_params() + 2 // plus 2 since stack.width in convention is at least 2
    }

    fn get_num_stack_params(&self) -> usize {
        match self {
            Opcode::Stop => 0,
            Opcode::Add => 2, // 2 params for adding
            Opcode::Sub => 2, // 2 params for subtracting
            Opcode::Mul => 2, // 2 params for multiplying
            Opcode::Div => 2, // 2 params for dividing
            Opcode::Mod => 2, // 2 params for dividing
            Opcode::Push4 => 0, // no param required
            Opcode::Dup2 => 2, // 2 params required
            Opcode::Pop => 1, // 1 param for popping
            Opcode::Return => 1, // 1 param for returning
            Opcode::Swap1 => 2, // 2 params for swapping
            Opcode::Jump => 1, // 1 param for pc to jump to the required destination
            Opcode::Jumpi => 2, // 2 params for condition and destination
            Opcode::Error => 1, // 1 param indicating error code
        }
    }
}

// this struct is used to put inside program memory
// it includes the opcode and possibly an additional param (like push value)
#[derive(Clone)]
pub struct OpcodeWithParams {
    opcode: Opcode,
    param: Option<u32>, // Some(param as u32) if there is some param. Otherwise, None
}

impl OpcodeWithParams {
    pub fn new(opcode: Opcode, param: Option<u32>) -> Self {
        Self {
            opcode: opcode,
            param: param,
        }
    }

    pub fn get_opcode(&self) -> Opcode {
        self.opcode.clone()
    }

    pub fn get_param(&self) -> Option<u32> {
        self.param
    }
}