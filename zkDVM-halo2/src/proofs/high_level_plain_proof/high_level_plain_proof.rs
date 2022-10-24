use strum::IntoEnumIterator;

use crate::{dummy_virtual_machine::{
    raw_execution_trace::RawExecutionTrace,
    opcode::Opcode
}, proofs::{deterministic_computations::program_counter_move_computation::compute_next_program_counter, proof_types::{p_opcode::POpcode, p_stack_depth::PStackDepth, p_program_counter::PProgramCounter, p_stack_value::PStackValue, p_location::PLocation, p_time_tag::PTimeTag, p_numeric_encoding::PNumericEncoding}}, utils::numeric_encoding::NumericEncoding};


struct HighLevelPlainProof {
    execution_trace: Vec<(PLocation, PTimeTag, POpcode, PStackValue)>, // (location, time_tag, opcode, value of corresponding stack location)
    lookup_table: Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode, PProgramCounter)>, // (stack_depth, program_counter, read_stack_value_1, read_stack_value_2, opcode, next_program_counter)
}

impl HighLevelPlainProof {
    fn new(execution_trace: &RawExecutionTrace) -> Self {
        Self {
            execution_trace: Self::extract_stack_trace_u32(execution_trace),
            lookup_table: Self::arrange_lookup_table(execution_trace),
        }
    }

    fn extract_stack_trace_u32(execution_trace: &RawExecutionTrace) -> Vec<(PLocation, PTimeTag, POpcode, PStackValue)> {
        execution_trace.get_stack_trace().iter().map(|stack_access| {
            (
                PLocation::from_u32(stack_access.get_location() as u32), 
                PTimeTag::from_u32(stack_access.get_time_tag()), 
                POpcode::from_u32(stack_access.get_access_operation().to_u32()),
                PStackValue::from_u32(stack_access.get_value()),
            )
        }).collect()
    }

    fn arrange_computation_table(execution_trace: &RawExecutionTrace) -> Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode)> {
        let depth_trace_len = execution_trace.get_depth_trace().len();
        let program_counter_trace_len = execution_trace.get_program_counter_trace().len();
        let stack_trace_len = execution_trace.get_stack_trace().len();
        let opcode_trace_len = execution_trace.get_opcode_trace().len();

        assert_eq!(program_counter_trace_len, depth_trace_len); // they must be equal
        assert_eq!(program_counter_trace_len * 3, stack_trace_len + 3); // stack_trace_len == (program_counter_trace_len - 1) * 3
        assert_eq!(program_counter_trace_len, opcode_trace_len + 1);

        let mut res: Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode)> = (0..opcode_trace_len).map(|index| {
            (
                PStackDepth::from_u32(execution_trace.get_depth_trace()[index] as u32), // depth before computing opcode
                PProgramCounter::from_u32(execution_trace.get_program_counter_trace()[index] as u32), // program counter before computing opcode
                // partitioning stack trace into tuple of 3 elements with corresponding AccessOperation sequence (Read, Read, Write)
                PStackValue::from_u32(execution_trace.get_stack_trace()[index * 3].get_value()), // then get first element with Read
                PStackValue::from_u32(execution_trace.get_stack_trace()[index * 3 + 1].get_value()), // the get second element with Read
                POpcode::from_u32(execution_trace.get_opcode_trace()[index].to_u32()), // extract the opcode
            )
        }).collect();

        let last_index = opcode_trace_len;
        res.push((
            PStackDepth::from_u32(execution_trace.get_depth_trace()[last_index] as u32), // get last depth of depth_trace
            PProgramCounter::from_u32(execution_trace.get_program_counter_trace()[last_index] as u32), // last pc of pc_trace
            PStackValue::from_u32(0), // no read value needed
            PStackValue::from_u32(0), // no read value needed
            POpcode::from_u32(0), // no opcode needed
        ));
        res
    }

    fn arrange_lookup_table(execution_trace: &RawExecutionTrace) -> Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode, PProgramCounter)> {
        let program_memory_length = execution_trace.get_program_memory().get_length() as u32;
        let error_index = execution_trace.get_program_memory().get_error_index() as u32;
        let stop_index = execution_trace.get_program_memory().get_stop_index() as u32;
        let opcode_trace_length = execution_trace.get_opcode_trace().len();

        // take the cartesian product of indices and all possible opcodes
        (0..opcode_trace_length).map(|index| {
            Opcode::iter().map(move |opcode| (index, opcode))
        }).flatten().map(|(index, opcode)| { // then for each of then, compute the corresponding tuple of elements
            let current_stack_depth = execution_trace.get_depth_trace()[index] as u32;
            let current_program_counter = execution_trace.get_program_counter_trace()[index] as u32; // current program counter
            let read_stack_value_1 = execution_trace.get_stack_trace()[index * 3].get_value(); // then get first element with Read
            let read_stack_value_2 = execution_trace.get_stack_trace()[index * 3 + 1].get_value(); // the get second element with Read
            (
                PStackDepth::from_u32(current_stack_depth),
                PProgramCounter::from_u32(current_program_counter),
                PStackValue::from_u32(read_stack_value_1),
                PStackValue::from_u32(read_stack_value_2),
                POpcode::from_u32(opcode.to_u32()), // current opcode
                PProgramCounter::from_u32(compute_next_program_counter(
                    current_stack_depth,
                    current_program_counter,
                    read_stack_value_1,
                    read_stack_value_2,
                    opcode.to_u32(),
                    program_memory_length,
                    error_index,
                    stop_index,
                )),
            )
        }).collect()
    }
}