use strum::{IntoEnumIterator, EnumCount};

use crate::{dummy_virtual_machine::{
    raw_execution_trace::RawExecutionTrace,
    opcode::Opcode, read_write_access::ReadWriteAccess, constants::{MAXIMUM_NUM_WRITES_PER_OPCODE, MAXIMUM_NUM_READS_PER_OPCODE, MAXIMUM_NUM_ACCESSES_PER_OPCODE},
}, proofs::{
    deterministic_computations::next_state_computation::compute_next_state, 
    proof_types::{
        p_opcode::POpcode, 
        p_stack_depth::PStackDepth, 
        p_program_counter::PProgramCounter, 
        p_stack_value::PStackValue,
        p_time_tag::PTimeTag, 
        p_numeric_encoding::PNumericEncoding, 
        p_read_write_acces::PReadWriteAccess, 
        p_program_memory_location::PProgramMemoryLocation, p_stack_location::PStackLocation
    }
}, utils::numeric_encoding::NumericEncoding
};


pub struct HighLevelPlainProof {
    num_transitions: usize, // number of transitions
    program_memory_length: usize, // length of program memory
    error_index: PProgramMemoryLocation,
    stop_index: PProgramMemoryLocation,


    stack_access_table: Vec<(PStackLocation, PTimeTag, PReadWriteAccess, PStackValue)>, // (location, time_tag, opcode, value of corresponding stack location) read from access value
    state_transition_table: Vec<(
        PStackDepth, // current stack depth before executing opcode
        PProgramCounter, // current program counter before executing opcode
        [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE],  // read_stack_value_1, read_stack_value_2, read_stack_3
        POpcode // opcode to execute
    )>,
    state_transition_lookup_table: Vec<(
        PStackDepth, // current stack depth before executing opcode
        PProgramCounter, // current program counter before executing opcode
        [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], // read_stack_values
        POpcode, // opcode to execute
        PStackDepth, // next_stack_depth
        PProgramCounter, // next program counter
        // [PStackValue; MAXIMUM_NUM_WRITES_PER_OPCODE], // write_stack_values
    )>, 
}

impl HighLevelPlainProof {
    pub fn new(execution_trace: &RawExecutionTrace) -> Self {
        Self {
            num_transitions: execution_trace.get_opcode_trace().len(),
            program_memory_length: execution_trace.get_program_memory().get_length(),
            error_index: PProgramMemoryLocation::from_u32(execution_trace.get_program_memory().get_error_index() as u32),
            stop_index: PProgramMemoryLocation::from_u32(execution_trace.get_program_memory().get_stop_index() as u32),
            stack_access_table: Self::extract_stack_access_table(execution_trace),
            state_transition_table: Self::arrange_state_transition_table(execution_trace),
            state_transition_lookup_table: Self::arrange_state_transition_lookup_table(execution_trace),
        }
    }

    fn extract_stack_access_table(execution_trace: &RawExecutionTrace) -> Vec<(PStackLocation, PTimeTag, PReadWriteAccess, PStackValue)> {
        // write to stack at inaccessible locations dummy values
        let mut res: Vec<(PStackLocation, PTimeTag, PReadWriteAccess, PStackValue)> = (0..MAXIMUM_NUM_READS_PER_OPCODE).map(|index|
            (
                PStackLocation::from_u32(index as u32),
                PTimeTag::from_u32(index as u32),
                PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32()),
                PStackValue::from_u32(0),
            )
        ).collect();

        res.extend(
            execution_trace.get_stack_trace().iter().map(|stack_access| {
                (
                    PStackLocation::from_u32(stack_access.get_location() as u32), 
                    PTimeTag::from_u32(stack_access.get_time_tag()), 
                    PReadWriteAccess::from_u32(stack_access.get_access_operation().to_u32()),
                    PStackValue::from_u32(stack_access.get_value()),
                )
            }).collect::<Vec<(PStackLocation, PTimeTag, PReadWriteAccess, PStackValue)>>()
        );
        res
    }

    fn arrange_state_transition_table(execution_trace: &RawExecutionTrace) -> Vec<(PStackDepth, PProgramCounter, [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], POpcode)> {
        let depth_trace_len = execution_trace.get_depth_trace().len();
        let program_counter_trace_len = execution_trace.get_program_counter_trace().len();
        let stack_trace_len = execution_trace.get_stack_trace().len();
        let opcode_trace_len = execution_trace.get_opcode_trace().len();

        assert_eq!(program_counter_trace_len, depth_trace_len); // they must be equal
        assert_eq!(opcode_trace_len * MAXIMUM_NUM_ACCESSES_PER_OPCODE, stack_trace_len); // stack_trace_len == opcode_trace_len * MAXIMUM_NUM_ACCESSES_PER_OPCODE
        assert_eq!(program_counter_trace_len, opcode_trace_len + 1);

        let mut res: Vec<(PStackDepth, PProgramCounter, [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], POpcode)> = (0..opcode_trace_len).map(|index| {
            (
                PStackDepth::from_u32(execution_trace.get_depth_trace()[index] as u32), // depth before computing opcode
                PProgramCounter::from_u32(execution_trace.get_program_counter_trace()[index] as u32), // program counter before computing opcode
                // partitioning stack trace into tuple of 3 elements with corresponding AccessOperation sequence (Read, Read, Write)
                {
                    let mut to_be_return_values = [PStackValue::from_u32(0); MAXIMUM_NUM_READS_PER_OPCODE];
                    for i in 0..MAXIMUM_NUM_READS_PER_OPCODE {
                        to_be_return_values[i] = PStackValue::from_u32(execution_trace.get_stack_trace()[index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i].get_value());
                    }
                    to_be_return_values
                },
                POpcode::from_u32(execution_trace.get_opcode_trace()[index].to_u32()), // extract the opcode
            )
        }).collect();

        let last_index = opcode_trace_len;
        res.push((
            PStackDepth::from_u32(execution_trace.get_depth_trace()[last_index] as u32), // get last depth of depth_trace
            PProgramCounter::from_u32(execution_trace.get_program_counter_trace()[last_index] as u32), // last pc of pc_trace
            {
                let mut last_read_array = [PStackValue::from_u32(0); MAXIMUM_NUM_READS_PER_OPCODE];
                for i in 0..MAXIMUM_NUM_WRITES_PER_OPCODE {
                    last_read_array[i] = PStackValue::from_u32(execution_trace.get_stack_trace()[(last_index - 1) * MAXIMUM_NUM_ACCESSES_PER_OPCODE + MAXIMUM_NUM_READS_PER_OPCODE + i].get_value());
                }
                last_read_array
            },
            POpcode::from_u32(0), // no opcode needed
        ));
        res
    }

    fn arrange_state_transition_lookup_table(execution_trace: &RawExecutionTrace) 
    -> Vec<(PStackDepth, PProgramCounter, [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], POpcode, PStackDepth, PProgramCounter)> {
        let program_memory_length = execution_trace.get_program_memory().get_length() as u32;
        let error_index = PProgramMemoryLocation::from_u32(execution_trace.get_program_memory().get_error_index() as u32);
        let stop_index = PProgramMemoryLocation::from_u32(execution_trace.get_program_memory().get_stop_index() as u32);
        let opcode_trace_length = execution_trace.get_opcode_trace().len();

        // take the cartesian product of indices and all possible opcodes
        (0..opcode_trace_length).map(|index| {
            Opcode::iter().map(move |opcode| (index, opcode))
        }).flatten().map(|(index, opcode)| { // then for each of then, compute the corresponding tuple of elements
            let current_stack_depth = PStackDepth::from_u32(execution_trace.get_depth_trace()[index] as u32);
            let current_program_counter = PProgramCounter::from_u32(execution_trace.get_program_counter_trace()[index] as u32); // current program counter
            let read_stack_values = {
                let mut to_be_assigned_array = [PStackValue::from_u32(0); MAXIMUM_NUM_READS_PER_OPCODE];
                for i in 0..MAXIMUM_NUM_READS_PER_OPCODE {
                    to_be_assigned_array[i] = PStackValue::from_u32(execution_trace.get_stack_trace()[index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i].get_value())
                }

                to_be_assigned_array
            };

            let (next_stack_depth, next_program_counter) = compute_next_state(
                &current_stack_depth,
                &current_program_counter,
                &read_stack_values,
                &POpcode::from_u32(opcode.to_u32()),
                program_memory_length,
                &error_index,
                &stop_index,
            );

            (
                current_stack_depth,
                current_program_counter,
                read_stack_values,
                POpcode::from_u32(opcode.to_u32()), // current opcode
                next_stack_depth,
                next_program_counter,
                // write_stack_values,
            )
        }).collect::<Vec<(PStackDepth, PProgramCounter, [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], POpcode, PStackDepth, PProgramCounter)>>()
    }


    // functions here are for verifying

    // verify stack_access_table
    fn verify_stack_access_table(&self) {
        print!("Do verify stack access table: ");
        // verify order of access of initial elements
        for index in 0..MAXIMUM_NUM_WRITES_PER_OPCODE {
            assert_eq!(self.stack_access_table[index].0.to_u32(), index as u32);
            assert_eq!(self.stack_access_table[index].1.to_u32(), index as u32);
            assert_eq!(self.stack_access_table[index].2, PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32()));
        }

        // verify order of access of remaining elements
        for index in 0..self.num_transitions {
            // verify correct access operations
            for i in 0..MAXIMUM_NUM_READS_PER_OPCODE {
                assert_eq!(
                    self.stack_access_table[MAXIMUM_NUM_READS_PER_OPCODE + index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i].2, 
                    PReadWriteAccess::from_u32(ReadWriteAccess::Read.to_u32())
                );
            }

            for i in MAXIMUM_NUM_READS_PER_OPCODE..MAXIMUM_NUM_ACCESSES_PER_OPCODE {
                assert_eq!(
                    self.stack_access_table[MAXIMUM_NUM_READS_PER_OPCODE + index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i].2, 
                    PReadWriteAccess::from_u32(ReadWriteAccess::Write.to_u32())
                );
            }

            // verify correct location access
            // todo!();
            for i in 1..MAXIMUM_NUM_READS_PER_OPCODE {
                assert_eq!(
                    self.stack_access_table[MAXIMUM_NUM_READS_PER_OPCODE + index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i - 1].0.to_u32(),     // top
                    self.stack_access_table[MAXIMUM_NUM_READS_PER_OPCODE + index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i].0.to_u32() + 1  // prev
                );
            }

            for i in MAXIMUM_NUM_READS_PER_OPCODE + 1..MAXIMUM_NUM_ACCESSES_PER_OPCODE {
                assert_eq!(
                    self.stack_access_table[MAXIMUM_NUM_READS_PER_OPCODE + index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i - 1].0.to_u32(),     // top
                    self.stack_access_table[MAXIMUM_NUM_READS_PER_OPCODE + index * MAXIMUM_NUM_ACCESSES_PER_OPCODE + i].0.to_u32() + 1  // prev
                );
            }
        }

        // verify correct time tag
        for index in 0..self.stack_access_table.len() {
            assert_eq!(self.stack_access_table[index].1.to_u32(), index as u32);
        }

        // verify sorting version
        let mut ordered_stack_access_table = self.stack_access_table.clone();
        ordered_stack_access_table.sort();
        for index in 0..ordered_stack_access_table.len() - 1 {
            let (cur_location, cur_time_tag, _, cur_stack_value) = &ordered_stack_access_table[index];
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
        
        // verify correct opcode setting
        for index in 0..self.num_transitions - 1 {
            for (rindex, opcode) in Opcode::iter().enumerate() {
                assert_eq!(self.state_transition_lookup_table[index * Opcode::COUNT + rindex].3, POpcode::from_u32(opcode.to_u32()));
            }
        }

        // verify correct next program counter
        for (
            stack_depth, 
            program_counter, 
            read_stack_values,
            opcode, 
            next_stack_depth, 
            next_program_counter,
            // write_stack_values,
        ) in &self.state_transition_lookup_table {
            let (computed_next_stack_depth, computed_next_program_counter) = compute_next_state(
                &stack_depth, 
                &program_counter, 
                read_stack_values,
                opcode, 
                self.program_memory_length as u32, 
                &self.error_index, 
                &self.stop_index,
            );

            assert_eq!(&computed_next_stack_depth, next_stack_depth);
            assert_eq!(&computed_next_program_counter, next_program_counter);
        }

        println!("succeed!");
    }

    fn is_tuple_inside_state_transition_lookup_table(&self, tuple: &(
        PStackDepth, 
        PProgramCounter, 
        [PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], 
        POpcode, 
        PStackDepth, 
        PProgramCounter, 
        // [PStackValue; MAXIMUM_NUM_WRITES_PER_OPCODE]
    )) -> bool {
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
            let (stack_depth, program_counter, read_stack_values, opcode) = &self.state_transition_table[index].clone();
            assert!(
                self.is_tuple_inside_state_transition_lookup_table(
                    &(
                        stack_depth.clone(), 
                        program_counter.clone(), 
                        read_stack_values.clone(),
                        opcode.clone(), 
                        self.state_transition_table[index + 1].0.clone(),
                        self.state_transition_table[index + 1].1.clone(),
                        // {
                        //     let mut write_stack_values = [PStackValue::from_u32(0); MAXIMUM_NUM_WRITES_PER_OPCODE];
                        //     for i in 0..MAXIMUM_NUM_WRITES_PER_OPCODE {
                        //         write_stack_values[i] = self.state_transition_table[index + 1].2[i]
                        //     }
                        //     write_stack_values
                        // },
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