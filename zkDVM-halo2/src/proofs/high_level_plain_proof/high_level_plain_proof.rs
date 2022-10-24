use crate::dummy_virtual_machine::{
    raw_execution_trace::RawExecutionTrace,
    numeric_encoding::NumericEncoding
};


struct HighLevelPlainProof {
    stack_trace_u32: Vec<(u32, u32, u32, u32)>, // (location, time_tag, opcode, value of corresponding stack location)
}

impl HighLevelPlainProof {
    fn new(execution_trace: &RawExecutionTrace) -> Self {
        Self {
            stack_trace_u32: Self::extract_stack_trace_u32(execution_trace),
        }
    }

    fn extract_stack_trace_u32(execution_trace: &RawExecutionTrace) -> Vec<(u32, u32, u32, u32)> {
        execution_trace.get_stack_trace().iter().map(|stack_access| {
            (
                stack_access.get_location() as u32, 
                stack_access.get_time_tag(), 
                stack_access.get_access_operation().to_u32(), 
                stack_access.get_value(),
            )
        }).collect()
    }

    fn arrange_computation_table(execution_trace: &RawExecutionTrace) -> Vec<(u32, u32, u32, u32, u32)> {
        let depth_trace_len = execution_trace.get_depth_trace().len();
        let program_counter_trace_len = execution_trace.get_program_counter_trace().len();
        let stack_trace_len = execution_trace.get_stack_trace().len();
        let opcode_trace_len = execution_trace.get_opcode_trace().len();

        assert_eq!(program_counter_trace_len, depth_trace_len); // they must be equal
        assert_eq!(program_counter_trace_len * 3, stack_trace_len + 3); // stack_trace_len == (program_counter_trace_len - 1) * 3
        assert_eq!(program_counter_trace_len, opcode_trace_len + 1);

        let mut res: Vec<(u32, u32, u32, u32, u32)> = (0..opcode_trace_len).map(|index| {
            (
                execution_trace.get_depth_trace()[index] as u32, // depth before computing opcode
                execution_trace.get_program_counter_trace()[index] as u32, // program counter before computing opcode
                // partitioning stack trace into tuple of 3 elements with corresponding AccessOperation sequence (Read, Read, Write)
                execution_trace.get_stack_trace()[index * 3].get_value(), // then get first element with Read
                execution_trace.get_stack_trace()[index * 3 + 1].get_value(), // the get second element with Read
                execution_trace.get_opcode_trace()[index].to_u32(), // extract the opcode
            )
        }).collect();

        let last_index = opcode_trace_len;
        res.push((
            execution_trace.get_depth_trace()[last_index] as u32, // get last depth of depth_trace
            execution_trace.get_program_counter_trace()[last_index] as u32, // last pc of pc_trace
            0, // no read value needed
            0, // no read value needed
            0, // no opcode needed
        ));
        res
    }
}