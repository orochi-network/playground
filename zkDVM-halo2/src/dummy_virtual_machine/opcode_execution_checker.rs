use super::{program_memory::ProgramMemory, error_code::ErrorCode};

pub trait OpcodeExecutionChecker {
    fn get_error_after_executing(&self, 
        read_stack_value_1: u32, 
        read_stack_value_2: u32, 
        program_memory_before_executing: &ProgramMemory,
        program_counter: usize,
    ) -> ErrorCode;
}

