use super::{program_memory::ProgramMemory, error_code::ErrorCode};

pub trait OpcodeExecutionChecker {
    fn get_error_after_executing(&self, 
        read_access_value_1: u32, 
        read_access_value_2: u32, 
        program_memory_before_executing: &ProgramMemory,
        program_counter: usize,
    ) -> ErrorCode;
}

