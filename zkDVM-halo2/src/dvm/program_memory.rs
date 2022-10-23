use super::opcode_definition::{Opcode, OpcodeWithParams};

#[derive(Clone)]
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

    pub fn get_opcode_with_params(&self, index: usize) -> OpcodeWithParams {
        self.memory[index]
    }

    pub fn is_program_counter_reasonable(&self) -> bool {
        self.program_counter < self.get_length()
    }

    // set program counter
    pub fn set_program_counter(&self, program_counter: usize) {
        self.program_counter = program_counter;
    }

    pub fn is_custom_program_counter_reasonable(&self, program_counter: usize) -> bool {
        program_counter < self.get_length()
    }

    pub fn display(&self) {
        println!();
        println!("------------- :@) ----------");
        println!("The following is program memory generated: ");
        for i in 0..self.memory.len() {
            println!("{i}. {:?}  {}", self.memory[i].get_opcode(), match self.memory[i].get_param() {
                Some(v) => v.to_string(),
                None => "".to_string(),
            });
        }
        println!();
        println!("Error index is {}", self.get_error_index());
        println!("Stop index is {}", self.get_stop_index());
        println!("------------- :@) ----------");
    }

    pub fn get_program_counter(&self) -> usize {
        self.program_counter
    }
    
}