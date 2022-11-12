use crate::runtime::constants::MAXIMUM_NUM_ACCESSES_PER_OPCODE;
use crate::runtime::error_code_util::error_code::ErrorCode;
use crate::runtime::program_memory_util::program_memory::ProgramMemory;

pub trait OpcodeExecutionChecker {
    fn get_error_after_executing(&self, 
        access_values: &[u32; MAXIMUM_NUM_ACCESSES_PER_OPCODE],
        program_memory: &ProgramMemory,
        program_counter_before_executing: usize,
    ) -> ErrorCode;
}

