use strum::{IntoEnumIterator, EnumCount};

use crate::{dummy_virtual_machine::{
    raw_execution_trace::RawExecutionTrace,
    opcode::Opcode, read_write_access::ReadWriteAccess, stack::Stack,
}, proofs::{
    deterministic_computations::next_state_computation::compute_next_state, 
    proof_types::{
        p_opcode::POpcode, 
        p_stack_depth::PStackDepth, 
        p_program_counter::PProgramCounter, 
        p_stack_value::PStackValue, 
        p_location::PLocation, 
        p_time_tag::PTimeTag, 
        p_numeric_encoding::PNumericEncoding, 
        p_read_write_acces::PReadWriteAccess, 
        p_tag::PTag
    }
}, utils::numeric_encoding::NumericEncoding
};


pub struct HighLevelPlainProof {
    num_transitions: usize, // number of transitions
    program_memory_length: usize, // length of program memory
    error_index: usize,
    stop_index: usize,


    stack_access_table: Vec<(PLocation, PTimeTag, PReadWriteAccess, PStackValue)>, // (location, time_tag, opcode, value of corresponding stack location) read from access value
    state_transition_table: Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode)>, // (stack_depth, program_counter, read_stack_value_1, read_stack_value_2, opcode)
    state_transition_lookup_table: Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode, PStackDepth, PProgramCounter)>, // (stack_depth, program_counter, read_stack_value_1, read_stack_value_2, opcode, next_program_counter)
}

impl HighLevelPlainProof {
    pub fn new(execution_trace: &RawExecutionTrace) -> Self {
        Self {
            num_transitions: execution_trace.get_opcode_trace().len(),
            program_memory_length: execution_trace.get_program_memory().get_length(),
            error_index: execution_trace.get_program_memory().get_error_index(),
            stop_index: execution_trace.get_program_memory().get_stop_index(),
            stack_access_table: Self::extract_stack_access_table(execution_trace),
            state_transition_table: Self::arrange_state_transition_table(execution_trace),
            state_transition_lookup_table: Self::arrange_state_transition_lookup_table(execution_trace),
        }
    }

    fn extract_stack_access_table(execution_trace: &RawExecutionTrace) -> Vec<(PLocation, PTimeTag, PReadWriteAccess, PStackValue)> {
        // write to stack at inaccessible locations dummy values
        let mut res: Vec<(PLocation, PTimeTag, PReadWriteAccess, PStackValue)> = (0..Stack::NUM_INACCESSIBLE_ELEMENTS).map(|index|
            (
                PLocation::from_u32(index as u32),
                PTimeTag::from_u32(index as u32),
                PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32()),
                PStackValue::from_u32(0),
            )
        ).collect();

        res.extend(
            execution_trace.get_stack_trace().iter().map(|stack_access| {
                (
                    PLocation::from_u32(stack_access.get_location() as u32), 
                    PTimeTag::from_u32(stack_access.get_time_tag()), 
                    PReadWriteAccess::from_u32(stack_access.get_access_operation().to_u32()),
                    PStackValue::from_u32(stack_access.get_value()),
                )
            }).collect::<Vec<(PLocation, PTimeTag, PReadWriteAccess, PStackValue)>>()
        );
        res
    }

    fn arrange_state_transition_table(execution_trace: &RawExecutionTrace) -> Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode)> {
        let depth_trace_len = execution_trace.get_depth_trace().len();
        let program_counter_trace_len = execution_trace.get_program_counter_trace().len();
        let stack_trace_len = execution_trace.get_stack_trace().len();
        let opcode_trace_len = execution_trace.get_opcode_trace().len();

        assert_eq!(program_counter_trace_len, depth_trace_len); // they must be equal
        assert_eq!(program_counter_trace_len * 4, stack_trace_len + 4); // stack_trace_len == (program_counter_trace_len - 1) * 4
        assert_eq!(program_counter_trace_len, opcode_trace_len + 1);

        let mut res: Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode)> = (0..opcode_trace_len).map(|index| {
            (
                PStackDepth::from_u32(execution_trace.get_depth_trace()[index] as u32), // depth before computing opcode
                PProgramCounter::from_u32(execution_trace.get_program_counter_trace()[index] as u32), // program counter before computing opcode
                // partitioning stack trace into tuple of 3 elements with corresponding AccessOperation sequence (Read, Read, Write)
                PStackValue::from_u32(execution_trace.get_stack_trace()[index * RawExecutionTrace::NUM_ACCESSES_PER_STEP].get_value()), // then get first element with Read
                PStackValue::from_u32(execution_trace.get_stack_trace()[index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 1].get_value()), // the get second element with Read
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

    fn arrange_state_transition_lookup_table(execution_trace: &RawExecutionTrace) -> Vec<(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode, PStackDepth, PProgramCounter)> {
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
            let read_stack_value_1 = execution_trace.get_stack_trace()[index * RawExecutionTrace::NUM_ACCESSES_PER_STEP].get_value(); // then get first element with Read
            let read_stack_value_2 = execution_trace.get_stack_trace()[index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 1].get_value(); // the get second element with Read

            let (next_stack_depth, next_program_counter) = compute_next_state(
                current_stack_depth,
                current_program_counter,
                read_stack_value_1,
                read_stack_value_2,
                opcode.to_u32(),
                program_memory_length,
                error_index,
                stop_index,
            );

            (
                PStackDepth::from_u32(current_stack_depth),
                PProgramCounter::from_u32(current_program_counter),
                PStackValue::from_u32(read_stack_value_1),
                PStackValue::from_u32(read_stack_value_2),
                POpcode::from_u32(opcode.to_u32()), // current opcode
                PStackDepth::from_u32(next_stack_depth),
                PProgramCounter::from_u32(next_program_counter),
            )
        }).collect()
    }


    // functions here are for verifying

    // verify stack_access_table
    fn verify_stack_access_table(&self) {
        print!("Do verify stack access table: ");
        // verify order of access of initial elements
        for index in 0..Stack::NUM_INACCESSIBLE_ELEMENTS {
            assert_eq!(self.stack_access_table[index].0.to_u32(), index as u32);
            assert_eq!(self.stack_access_table[index].1.to_u32(), index as u32);
            assert_eq!(self.stack_access_table[index].2, PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32()));
        }

        // verify order of access of remaining elements
        for index in 0..self.num_transitions {
            // verify correct access operations
            assert_eq!(
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 0].2, 
                PReadWriteAccess::from_u32(ReadWriteAccess::Read.to_u32())
            );
            assert_eq!(
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 1].2, 
                PReadWriteAccess::from_u32(ReadWriteAccess::Read.to_u32())
            );
            assert_eq!(
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 2].2, 
                PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32())
            );
            assert_eq!(
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 3].2, 
                PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32())
            );

            // verify correct location access
            assert_eq!(
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 0].0.to_u32(),     // top
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 1].0.to_u32() + 1  // prev
            );

            assert_eq!(
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 2].0.to_u32(),     // top
                self.stack_access_table[Stack::NUM_INACCESSIBLE_ELEMENTS + index * RawExecutionTrace::NUM_ACCESSES_PER_STEP + 3].0.to_u32() + 1  // prev
            );
        }

        // verify correct time tag
        for index in 0..self.stack_access_table.len() {
            assert_eq!(self.stack_access_table[index].1.to_u32(), index as u32);
        }

        // verify sorting version
        let mut ordered_stack_access_table = self.stack_access_table.clone();
        ordered_stack_access_table.sort();
        for index in 0..ordered_stack_access_table.len() - 1 {
            let (cur_location, cur_time_tag, cur_access_operation, cur_stack_value) = &ordered_stack_access_table[index];
            let (next_location, next_time_tag, next_access_operation, next_stack_value) = &ordered_stack_access_table[index + 1];

            assert!(
                // either current location is less than next location
                // or if current location == next location, current and next time tags must be different
                (cur_location < next_location || (cur_location == next_location && cur_time_tag < next_time_tag))
                // current location is different from next location
                // or if current location == next location, value must be the same,
                // of if current location == next location and value are different, next location must be a Write access
                && (cur_location != next_location || cur_stack_value == next_stack_value || *next_access_operation == PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32()))
                // current location is the same as next location
                // or if current location if different from next location, write access must be applied first
                && (cur_location == next_location || *next_access_operation == PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32()))
            );
        }
        println!("succeed!");
    }

    // verify state_transition_lookup_table
    fn verify_state_transition_lookup_table(&self) {
        print!("Do verify state transition lookup table: ");
        let num_state_transitions = self.state_transition_table.len();
        
        // verify correct opcode setting
        for index in 0..num_state_transitions - 1 {
            for (rindex, opcode) in Opcode::iter().enumerate() {
                assert_eq!(self.state_transition_lookup_table[index * Opcode::COUNT + rindex].4.to_u32(), opcode.to_u32());
            }
        }

        // verify correct next program counter
        for (
            stack_depth, 
            program_counter, 
            read_stack_value_1, 
            read_stack_value_2, 
            opcode, 
            next_stack_depth, 
            next_program_counter
        ) in &self.state_transition_lookup_table {
            let (computed_next_stack_depth, computed_next_program_counter) = compute_next_state(
                stack_depth.to_u32(), 
                program_counter.to_u32(), 
                read_stack_value_1.to_u32(), 
                read_stack_value_2.to_u32(), 
                opcode.to_u32(), 
                self.program_memory_length as u32, 
                self.error_index as u32, 
                self.stop_index as u32,
            );

            assert_eq!(computed_next_stack_depth, next_stack_depth.to_u32());
            assert_eq!(computed_next_program_counter, next_program_counter.to_u32());
        }

        println!("succeed!");
    }

    fn is_tuple_inside_state_transition_lookup_table(&self, tuple: &(PStackDepth, PProgramCounter, PStackValue, PStackValue, POpcode, PStackDepth, PProgramCounter)) -> bool {
        for element in &self.state_transition_lookup_table {
            if element == tuple {
                return true;
            }
        }
        false
    }

    // verify state transition table
    fn verify_state_transition_table(&self) {
        print!("Do verify state transition table: ");
        for index in 0..self.state_transition_table.len() - 1 {
            let (stack_depth, program_counter, read_stack_value_1, read_stack_value_2, opcode) = self.state_transition_table[index].clone();
            assert!(
                self.is_tuple_inside_state_transition_lookup_table(
                    &(
                        stack_depth, 
                        program_counter, 
                        read_stack_value_1, 
                        read_stack_value_2, 
                        opcode, 
                        self.state_transition_table[index + 1].0.clone(),
                        self.state_transition_table[index + 1].1.clone()
                    )
                )
            );
        }
        println!("succeed!");
    }

    pub fn verify(&self) {
        self.verify_stack_access_table();
        self.verify_state_transition_lookup_table();
        self.verify_state_transition_table();
    }
}