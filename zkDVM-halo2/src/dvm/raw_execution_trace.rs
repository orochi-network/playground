use std::fs::read;

use super::{program_memory::ProgramMemory, direction::Direction, stack_access::StackAccess, read_write_access::ReadWriteAccess};

pub struct RawExecutionTrace {
    program_memory: ProgramMemory, // public: store the sequence of opcodes (encoded into u32)
    direction_trace: Vec<Direction>, // recording sequence of bits for directing next program pc 
    program_counter_trace: Vec<usize>, // advice: store pc after each execution
    stack_trace: Vec<StackAccess>,//
    // opcode_trace: Vec<Opcode>, // advice: store the encoded opcodes (u32) according to pc_trace
    // depth_trace: Vec<usize>, // advice: store the top pointer of the stack
    // lhs_trace: Vec<u32>, // advice: each lhs as input of each opcode
    // rhs_trace: Vec<u32>, // advice: each rhs as input of each opcode
    // out_trace: Vec<u32>, // advice: each out as output of each opcode
}

impl RawExecutionTrace {
    pub fn new(program_memory: ProgramMemory, initial_program_counter: usize) -> Self {
        Self {
            program_memory: program_memory,
            direction_trace: Vec::<Direction>::new(),
            program_counter_trace: vec![initial_program_counter],
            stack_trace: Vec::<StackAccess>::new(),
            // program_counter_trace: Vec::<usize>::new(),
            // opcode_trace: Vec::<Opcode>::new(),
            // depth_trace: Vec::<usize>::new(),
        }
    }
    
    pub fn push(&mut self, 
        direction: Direction, 
        program_counter: usize, 
        time_tag: &mut u32,
        depth_before_changed: usize,
        read_access_value_1: u32,
        read_access_value_2: u32,
        depth_after_changed: usize,
        write_access_value: u32,
    ) {
        self.direction_trace.push(direction);
        self.program_counter_trace.push(program_counter);

        [
            (depth_before_changed - 1, time_tag.clone(), ReadWriteAccess::Read, read_access_value_1),
            (depth_before_changed - 2, time_tag.clone() + 1, ReadWriteAccess::Read, read_access_value_2),
            (depth_after_changed - 1, time_tag.clone() + 2, ReadWriteAccess::Write, write_access_value),
        ].map(|(location, time_tag, access_operation, value)| {
                self.stack_trace.push(
                    StackAccess::new(
                        location,
                        time_tag.clone(),
                        access_operation,
                        value,
                    )
                );
            }
        );
        *time_tag += 3; // increase time_tag by 3 for 2 READ and 1 WRITE access
    }

    pub fn get_direction_trace(&self) -> &Vec<Direction> {
        &self.direction_trace
    }

    pub fn get_program_counter_trace(&self) -> &Vec<usize> {
        &self.program_counter_trace
    }
}