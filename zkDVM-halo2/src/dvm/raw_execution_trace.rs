use super::{program_memory::ProgramMemory, opcode_definition::Opcode, direction::Direction};

pub struct RawExecutionTrace {
    program_memory: ProgramMemory, // public: store the sequence of opcodes (encoded into u32)
    direction_trace: Vec<Direction>, // recording sequence of bits for directing next program pc 
    program_counter_trace: Vec<usize>, // advice: store pc after each execution
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
            // program_counter_trace: Vec::<usize>::new(),
            // opcode_trace: Vec::<Opcode>::new(),
            // depth_trace: Vec::<usize>::new(),
        }
    }
    
    pub fn push(&mut self, direction: Direction, program_counter: usize) {
        self.direction_trace.push(direction);
        self.program_counter_trace.push(program_counter);
    }

    pub fn get_direction_trace(&self) -> &Vec<Direction> {
        &self.direction_trace
    }

    pub fn get_program_counter_trace(&self) -> &Vec<usize> {
        &self.program_counter_trace
    }
}