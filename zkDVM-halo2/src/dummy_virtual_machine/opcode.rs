use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::utils::numeric_encoding::NumericEncoding;

use super::{
    stack::Stack, 
    opcode_execution_checker::OpcodeExecutionChecker, 
    program_memory::ProgramMemory, 
    stack_requirement::StackRequirement, 
    error_code::ErrorCode, 
    opcode_with_params::OpcodeWithParams
};

#[derive(Clone, PartialEq, Eq, FromPrimitive, Debug, EnumIter, PartialOrd, Ord)]
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

impl NumericEncoding for Opcode {
    fn to_u32(&self) -> u32 {
        *self as u32
    }

    fn from_u32(v: u32) -> Self {
        FromPrimitive::from_u32(v).unwrap()
    }
}

impl StackRequirement for Opcode {
    fn get_minimum_stack_depth(&self) -> usize {
        self.get_num_stack_params() + Stack::NUM_INACCESSIBLE_ELEMENTS // plus 2 since stack.width in convention is at least 2
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

impl OpcodeExecutionChecker for Opcode {
    fn get_error_after_executing(&self, 
        read_stack_value_1: u32, 
        read_stack_value_2: u32, 
        program_memory_before_executing: &ProgramMemory,
        program_counter_before_executing: usize,
    ) -> ErrorCode {
        match self {
            Opcode::Stop | Opcode::Return | Opcode::Error => {
                // Stop does not move pc => NoError
                // Return and Error move pc to location of Stop => always NoError
                
                ErrorCode::NoError
            },
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
                // pc is moved next
                match program_memory_before_executing.is_program_counter_reasonable(
                    program_counter_before_executing + 1
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
            Opcode::Div | Opcode::Mod => {
                let (a, b) = (read_stack_value_1, read_stack_value_2);
                match b {
                    0 => ErrorCode::DivisionByZero,
                    _ => {
                        // pc is moved next
                        match program_memory_before_executing.is_program_counter_reasonable(
                            program_counter_before_executing + 1
                        ) {
                            false => ErrorCode::IncorrectProgramCounter,
                            true => ErrorCode::NoError,
                        }
                    }
                }
            },
            Opcode::Jump => {
                let destination = read_stack_value_1;
                match program_memory_before_executing.is_program_counter_reasonable(
                    destination as usize
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
            Opcode::Jumpi => {
                let (destination, condition) = (read_stack_value_1, read_stack_value_2);

                // get next pc according to condition
                let next_program_counter = match condition {
                    0 => program_counter_before_executing + 1,
                    _ => destination as usize,
                };

                // then check validity of pc
                match program_memory_before_executing.is_program_counter_reasonable(
                    next_program_counter
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
        }
    }
}