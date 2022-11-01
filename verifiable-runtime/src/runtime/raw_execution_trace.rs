use super::{program_memory::ProgramMemory, constants::{MAXIMUM_NUM_WRITES_PER_OPCODE, MAXIMUM_NUM_READS_PER_OPCODE}, opcode_with_params::OpcodeWithParams, stack_access::StackAccess, access_operation::AccessOperation};

pub struct RawExecutionTrace {
    program_memory: ProgramMemory, // public: store the sequence of opcodes (encoded into u32) and never change in the future

    depth_trace: Vec<usize>, // advice: store the depth of the stack 
    program_counter_trace: Vec<usize>, // advice: store pc after each execution
    stack_trace: Vec<StackAccess>, // advice: store all possible accesses to stack with respective time, location, operation
    opcode_with_params_trace: Vec<OpcodeWithParams>, // advice: store the encoded opcodes (u32) according to pc_trace
}

impl RawExecutionTrace {

    pub fn new(program_memory: &ProgramMemory, initial_program_counter: usize) -> Self {
        Self {
            program_memory: program_memory.clone(), // TODO: recommending changing to reference with life time, to be fixed later
            
            program_counter_trace: vec![initial_program_counter], // initialized with the first program_counter
            stack_trace: Vec::<StackAccess>::new(),
            depth_trace: vec![MAXIMUM_NUM_READS_PER_OPCODE], // depth trace must have 1 element for initial stack
            opcode_with_params_trace: Vec::<OpcodeWithParams>::new(),
            // depth_trace: Vec::<usize>::new(),
        }
    }
    
    pub fn push(&mut self, 
        time_tag: &mut u32, // time_tag a mutable reference whose value is the latest time hasn't been assigned to any element in stack_trace
        depth_before_changed: usize,
        read_stack_values: [u32; MAXIMUM_NUM_READS_PER_OPCODE],
        opcode_with_params_for_current_execution: OpcodeWithParams,
        depth_after_changed: usize,
        program_counter_after_changed: usize, 
        write_stack_values: [u32; MAXIMUM_NUM_WRITES_PER_OPCODE],
    ) {
        self.depth_trace.push(depth_after_changed);
        self.program_counter_trace.push(program_counter_after_changed);

        for i in 0..MAXIMUM_NUM_READS_PER_OPCODE {
            self.stack_trace.push(
                StackAccess::new(
                    depth_before_changed - i - 1,
                    *time_tag,
                    AccessOperation::Read,
                    read_stack_values[i],
                )
            );
            *time_tag += 1;
        }

        for i in 0..MAXIMUM_NUM_WRITES_PER_OPCODE {
            self.stack_trace.push(
                StackAccess::new(
                    depth_after_changed - i - 1,
                    *time_tag,
                    AccessOperation::Write,
                    write_stack_values[i],
                )
            );
            *time_tag += 1;
        }

        self.opcode_with_params_trace.push(opcode_with_params_for_current_execution);
        
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

    pub fn get_opcode_with_params_trace(&self) -> &Vec<OpcodeWithParams> {
        &self.opcode_with_params_trace
    }

    pub fn get_program_memory(&self) -> &ProgramMemory {
        &&self.program_memory
    }
}