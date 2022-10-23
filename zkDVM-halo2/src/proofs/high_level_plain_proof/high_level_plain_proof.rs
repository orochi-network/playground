use crate::dummy_virtual_machine::{
    raw_execution_trace::RawExecutionTrace,
    numeric_encoding::NumericEncoding
};


// struct HighLevelPlainProof {
//     stack_trace_u32: Vec<(u32, u32, u32, u32)>,
// }

// impl HighLevelPlainProof {
//     fn new(execution_trace: &RawExecutionTrace) -> Self {
//         Self {
//             stack_trace_u32: Self::extract_stack_trace_u32(execution_trace),
//         }
//     }

//     fn extract_stack_trace_u32(execution_trace: &RawExecutionTrace) -> Vec<(u32, u32, u32, u32)> {
//         execution_trace.get_stack_trace().iter().map(|stack_access| {
//             (
//                 stack_access.get_location() as u32, 
//                 stack_access.get_time_tag(), 
//                 stack_access.get_access_operation().to_u32(), 
//                 stack_access.get_value(),
//             )
//         }).collect()
//     }

//     fn arrange_program_counter_correct_computation(execution_trace: &RawExecutionTrace) -> Vec<(u32, u32, u32, u32, u32)> {
//         let program_counter_trace_len = execution_trace.get_program_counter_trace().len();
//         let depth_trace_len = execution_trace.get_depth_trace().len();
//         let stack_trace_len = execution_trace.get_stack_trace().len();
//         let opcode_trace_len = execution_trace.get_opcode_trace().len();
//         assert_eq!(program_counter_trace_len, depth_trace_len);
//         assert_eq!(program_counter_trace_len, depth_trace_len);
//         assert_eq!((program_counter_trace_len - 1) * 3, stack_trace_len);

//         let res = (0..program_counter_trace_len - 1).map(|index| {
//             (
//                 execution_trace.get_program_counter_trace()[index],
//                 execution_trace.get_depth_trace()[index],
//                 execution_trace.get_stack_trace()[index * 3].get_value(),
//                 execution_trace.get_stack_trace()[index * 3 + 1].get_value(),
//                 execution_trace.get_opcode_trace()[index].to_u32(),
//             )
//         }).collect();
//     }
// }