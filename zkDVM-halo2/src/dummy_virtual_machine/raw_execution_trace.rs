use super::{program_memory::ProgramMemory, stack_access::StackAccess, read_write_access::ReadWriteAccess, stack::Stack, opcode::Opcode};

pub struct RawExecutionTrace {
    program_memory: ProgramMemory, // public: store the sequence of opcodes (encoded into u32) and never change in the future

    depth_trace: Vec<usize>, // advice: store the depth of the stack 
    program_counter_trace: Vec<usize>, // advice: store pc after each execution
    stack_trace: Vec<StackAccess>, // advice: store all possible accesses to stack with respective time, location, operation
    opcode_trace: Vec<Opcode>, // advice: store the encoded opcodes (u32) according to pc_trace
    // lhs_trace: Vec<u32>, // advice: each lhs as input of each opcode
    // rhs_trace: Vec<u32>, // advice: each rhs as input of each opcode
    // out_trace: Vec<u32>, // advice: each out as output of each opcode
}

impl RawExecutionTrace {
    pub const NUM_ACCESSES_PER_STEP: usize = 4;

    pub fn new(program_memory: &ProgramMemory, initial_program_counter: usize) -> Self {
        Self {
            program_memory: program_memory.clone(), // TODO: recommending changing to reference with life time, to be fixed later
            
            program_counter_trace: vec![initial_program_counter], // initialized with the first program_counter
            stack_trace: Vec::<StackAccess>::new(),
            depth_trace: vec![Stack::NUM_INACCESSIBLE_ELEMENTS], // depth trace must have 1 element for initial stack
            opcode_trace: Vec::<Opcode>::new(),
            // depth_trace: Vec::<usize>::new(),
        }
    }
    
    pub fn push(&mut self, 
        program_counter_after_changed: usize, 
        time_tag: &mut u32, // time_tag a mutable reference whose value is the latest time hasn't been assigned to any element in stack_trace
        depth_before_changed: usize,
        read_stack_value_1: u32,
        read_stack_value_2: u32,
        depth_after_changed: usize,
        write_stack_value_top: u32,  
        write_stack_value_prev: u32, 
        opcode_for_current_execution: Opcode,
    ) {
        self.depth_trace.push(depth_after_changed);
        self.program_counter_trace.push(program_counter_after_changed);

        [
            (depth_before_changed - 1, time_tag.clone(), ReadWriteAccess::Read, read_stack_value_1),
            (depth_before_changed - 2, time_tag.clone() + 1, ReadWriteAccess::Read, read_stack_value_2),
            (depth_after_changed - 1, time_tag.clone() + 2, ReadWriteAccess::Write, write_stack_value_top),
            (depth_after_changed - 2, time_tag.clone() + 3, ReadWriteAccess::Write, write_stack_value_prev),
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

        *time_tag += Self::NUM_ACCESSES_PER_STEP as u32; // increase time_tag by 3 for 2 READ and 1 WRITE access
        self.opcode_trace.push(opcode_for_current_execution);
    }

    pub fn get_program_counter_trace(&self) -> &Vec<usize> {
        &self.program_counter_trace
    }

    pub fn get_stack_trace(&self) -> &Vec<StackAccess> {
        &self.stack_trace
    }

    pub fn get_depth_trace(&self) -> &Vec<usize> {
        &self.depth_trace
    }

    pub fn get_opcode_trace(&self) -> &Vec<Opcode> {
        &self.opcode_trace
    }

    pub fn get_program_memory(&self) -> &ProgramMemory {
        &&self.program_memory
    }
}