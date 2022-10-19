use crate::opcode_definition::{Opcode, OpcodeWithParams};

pub struct ProgramMemory {
    memory: Vec<OpcodeWithParams>,
    program_counter: usize,
}

impl ProgramMemory {
    pub fn new(memory: Vec<OpcodeWithParams>) -> Self {
        // program must be executable
        assert!(memory.len() > 0);

        let mut new_memory = memory.clone();

        // each program must have an errorcode at the end to return whenever error happens
        new_memory.push(OpcodeWithParams::new(Opcode::Error, None));
        // then followed by Opcode::Stop
        new_memory.push(OpcodeWithParams::new(Opcode::Stop, None));
        // then return
        Self {
            memory: new_memory,
            program_counter: 0,
        }
    }

    // return number of elements of the program memory
    pub fn get_length(&self) -> usize {
        self.memory.len()
    }

    // return program_counter according to Opcode::Error
    pub fn get_error_index(&self) -> usize {
        self.memory.len() - 2
    }

    // return program_counter according to Opcode::Stop
    pub fn get_stop_index(&self) -> usize {
        self.memory.len() - 1
    }

    pub fn get_current_opcode_with_params(&self) -> OpcodeWithParams {
        self.memory[self.program_counter].clone()
    }

    pub fn is_program_counter_reasonable(&self) -> bool {
        self.program_counter < self.get_length()
    }

    pub fn next_program_counter(&mut self) {
        self.program_counter += 1;
    }

    pub fn next_program_counter_with_destination(&mut self, destination: usize) {
        self.program_counter = destination;
    }

    // pub fn get_program_counter_mut(&self) -> &mut usize {
    //     &mut self.program_counter
    // }
}