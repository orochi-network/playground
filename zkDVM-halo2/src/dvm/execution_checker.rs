use std::fs::read;

use super::{opcode_definition::{Opcode, ErrorCode}, program_memory::ProgramMemory};

pub trait ExecutionChecker {
    fn get_error_after_executing(&self, 
        read_access_value_1: u32, 
        read_access_value_2: u32, 
        program_memory_before_executing: &ProgramMemory
    ) -> ErrorCode;
}

impl ExecutionChecker for Opcode {
    fn get_error_after_executing(&self, 
        read_access_value_1: u32, 
        read_access_value_2: u32, 
        program_memory_before_executing: &ProgramMemory
    ) -> ErrorCode {
        match self {
            Opcode::Stop | Opcode::Return | Opcode::Error => {
                // Stop does not move pc => NoError
                // Return and Error move pc to location of Stop => always NoError
                
                ErrorCode::NoError
            },
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
                // pc is moved next
                match program_memory_before_executing.is_custom_program_counter_reasonable(
                    program_memory_before_executing.get_program_counter() + 1
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
            Opcode::Div | Opcode::Mod => {
                let (a, b) = (read_access_value_1, read_access_value_2);
                match b {
                    0 => ErrorCode::DivisionByZero,
                    _ => {
                        // pc is moved next
                        match program_memory_before_executing.is_custom_program_counter_reasonable(
                            program_memory_before_executing.get_program_counter() + 1
                        ) {
                            false => ErrorCode::IncorrectProgramCounter,
                            true => ErrorCode::NoError,
                        }
                    }
                }
            },
            Opcode::Jump => {
                let destination = read_access_value_1;
                match program_memory_before_executing.is_custom_program_counter_reasonable(
                    destination as usize
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
            Opcode::Jumpi => {
                let (destination, condition) = (read_access_value_1, read_access_value_2);

                // get next pc according to condition
                let next_next_program_counter = match condition {
                    0 => program_memory_before_executing.get_program_counter() + 1,
                    _ => destination as usize,
                };

                // then check validity of pc
                match program_memory_before_executing.is_custom_program_counter_reasonable(
                    next_next_program_counter
                ) {
                    false => ErrorCode::IncorrectProgramCounter,
                    true => ErrorCode::NoError,
                }
            },
        }
    }
}