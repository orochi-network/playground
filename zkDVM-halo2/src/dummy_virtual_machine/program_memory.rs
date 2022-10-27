use super::{
    opcode::Opcode,
    opcode_with_params::OpcodeWithParams,
};

use std::ops::Index;

#[derive(Clone)]
pub struct ProgramMemory {
    memory: Vec<OpcodeWithParams>,
}

impl ProgramMemory {
    pub fn new(memory: Vec<OpcodeWithParams>) -> Self {
        // program must be executable
        assert!(memory.len() > 0);

        let mut new_memory = memory.clone();

        // each program must have an errorcode at the end to return whenever error happens
        new_memory.push(OpcodeWithParams::new(Opcode::Error, &[None]));
        // then followed by Opcode::Stop
        new_memory.push(OpcodeWithParams::new(Opcode::Stop, &[None]));
        // then return
        Self {
            memory: new_memory,
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

    pub fn is_program_counter_reasonable(&self, program_counter: usize) -> bool {
        program_counter < self.get_length()
    }

    pub fn display(&self) {
        println!();
        println!("------------- :@) ----------");
        println!("The following is program memory generated: ");
        for i in 0..self.memory.len() {
            println!("{i}. {:?}  {:?}", self.memory[i].get_opcode(), self.memory[i].get_all_params());
        }
        println!();
        println!("Error index is {}", self.get_error_index());
        println!("Stop index is {}", self.get_stop_index());
        println!("------------- :@) ----------");
    }
    
}

impl Index<usize> for ProgramMemory {
    type Output = OpcodeWithParams;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}