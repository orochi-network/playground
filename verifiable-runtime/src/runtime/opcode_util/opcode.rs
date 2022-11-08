use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum_macros::{EnumIter, EnumCount};
use strum::IntoEnumIterator;
use crate::runtime::access_util::access_specficiation_extractor::AccessSpecificationExtractor;
use crate::runtime::access_util::access_specification::AccessSpecification;
use crate::runtime::access_util::access_type::AccessType;

use crate::runtime::constants::{MAXIMUM_NUM_READS_PER_OPCODE, MAXIMUM_NUM_WRITES_PER_OPCODE};
use crate::runtime::error_code_util::error_code::ErrorCode;
use crate::runtime::program_memory_util::program_memory::ProgramMemory;
use crate::runtime::stack_util::stack_requirement::StackRequirement;

use crate::utils::numeric_encoding::NumericEncoding;

// use super::{opcode_execution_checker::OpcodeExecutionChecker};

#[derive(Clone, PartialEq, Eq, FromPrimitive, Debug, EnumIter, PartialOrd, Ord, EnumCount, Copy)]
pub enum Opcode {
    Stop = 0x00, // depth is unchanged, program counter is unchanged too
    Add = 0x01, // depth is decreased by 1 with written output = lhs + rhs, pc += 1
    Sub = 0x02, // depth is decreased by 1 with written output = lhs - rhs, pc += 1
    Mul = 0x03, // depth is decreased by 1 with written output = lhs * rhs, pc += 1
    Div = 0x04, // depth is decreased by 1 with written output = lhs // rhs, if rhs == 0 then jump to ErrDivisionByZero, pc += 1
    Mod = 0x06, // depth is decreased by 1 with written output = lhs % rhs, if rhs == 0 then jump to ErrDivisionByZero, pc += 1
    Pop = 0x50, // depth is decreased by 1, no constraint to lhs and rhs, pc += 1
    Mload = 0x51, // depth is unchanged, pop offset, read from mem[offset], then push back to stack
    Mstore = 0x52, // depth is decreased by 2, pop offset then value and write mem[offset] = value
    Jump = 0x56, // depth is kept unchanged with written output = rhs, pc = new_pc
    Jumpi = 0x57, // depth is kept unchanged with written output = rhs, pc = (bool(rhs)) * (pc + 1) + (1 - bool(rhs)) * new_pc
    Push4 = 0x63, // depth is increased by 1, no constraint to lhs and rhs, pc += 1
    Dup1 = 0x80, // depth is increased by 1 with written output = stack[depth - 1], pc += 1
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
    fn get_num_read_write_params(&self) -> usize {
        match self {
            Opcode::Stop => 0, // no params => 0 RW in total
            Opcode::Add => 3, // pop 2 params from stack and push 1 param to stack => 3 RWs in total
            Opcode::Sub => 3, // pop 2 params from stack and push 1 param to stack => 3 RWs in total
            Opcode::Mul => 3, // pop 2 params from stack and push 1 param to stack => 3 RWs in total
            Opcode::Div => 3, // pop 2 params from stack and push 1 param to stack => 3 RWs in total
            Opcode::Mod => 3, // pop 2 params from stack and push 1 param to stack => 3 RWs in total
            Opcode::Push4 => 1, // push 1 immediate param to stack => 1 RW in total
            Opcode::Mload => 3, // read 1 param from stack, 1 param from mem and push to stack => 3 RWs in total
            Opcode::Mstore => 3, // pop 2 params from stack and write to mem => 3 RWs in total
            Opcode::Dup1 => 2, // read and push 1 param to stack => 2 RWs in total
            Opcode::Dup2 => 2, // read and push 1 param to stack => 2 RWs in total
            Opcode::Pop => 1, // pop 1 param from stack => 1 RW in total
            Opcode::Return => 1, // pop 1 param from stack => 1 RW in total
            Opcode::Swap1 => 4, // pop 2 params from stack and write 2 params => 4 RWs in total
            Opcode::Jump => 1, // 1 param for pc to jump to destination => 1 RW in total
            Opcode::Jumpi => 2, // 2 params for condition and destination => 2 RWs in total
            Opcode::Error => 1, // 1 param indicating error code => 1 RW in total
            // currently maximum = 4 RWs in total
        }
    }

    fn get_minimum_stack_depth(&self) -> usize {
        todo!()
    }
}

impl OpcodeExecutionChecker for Opcode {
    fn get_error_after_executing(&self,
        read_stack_values: &[u32; MAXIMUM_NUM_READS_PER_OPCODE],
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
                let b = &read_stack_values[1];
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
                let destination = &read_stack_values[0];
                match program_memory_before_executing.is_program_counter_reasonable(
                    *destination as usize
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
            Opcode::Jumpi => {
                let (destination, condition) = &(read_stack_values[0], read_stack_values[1]);

                // get next pc according to condition
                let next_program_counter = match condition {
                    0 => program_counter_before_executing + 1,
                    _ => *destination as usize,
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

impl AccessSpecificationExtractor for Opcode {
    fn get_access_specification(&self) -> (
        [AccessSpecification; MAXIMUM_NUM_READS_PER_OPCODE],
        [AccessSpecification; MAXIMUM_NUM_WRITES_PER_OPCODE]
    ) {
        match self {
            Opcode::Stop => {
                (
                    [
                        AccessSpecification::new(AccessType::Garbage, None),
                    ],
                    [

                    ]
                )
            },
            Opcode::Add => {},
            Opcode::Sub => {},
            Opcode::Mul => {},
            Opcode::Div => {},
            Opcode::Mod => {},
            Opcode::Pop => {},
            Opcode::Jump => {},
            Opcode::Jumpi => {},
            Opcode::Push4 => {},
            Opcode::Dup2 => {},
            Opcode::Swap1 => {},
            Opcode::Return => {},
            Opcode::Error => {},
        }
    }
}