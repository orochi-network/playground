use super::{program_memory::ProgramMemory, error_code::ErrorCode, constants::MAXIMUM_NUM_READS_PER_OPCODE};

pub trait OpcodeExecutionChecker {
    fn get_error_after_executing(&self, 
        read_stack_values: &[u32; MAXIMUM_NUM_READS_PER_OPCODE],
        program_memory_before_executing: &ProgramMemory,
        program_counter: usize,
    ) -> ErrorCode;
}

