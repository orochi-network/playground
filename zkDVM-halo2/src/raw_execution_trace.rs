struct OpcodeStructure {
    id: u32,
    first_param: u32,
    second_param: u32,
}

struct RawExecutionTrace {
    program_memory: Vec<OpcodeStructure>, // public: store the sequence of opcodes (encoded into u32)
    direction_trace: Vec<u8>, // recording sequence of bits for directing next program pc 
    program_counter_trace: Vec<u32>, // advice: store pc after each execution
    opcode_trace: Vec<u32>, // advice: store the encoded opcodes (u32) according to pc_trace
    top_trace: Vec<u32>, // advice: store the top pointer of the stack
    lhs_trace: Vec<u32>, // advice: each lhs as input of each opcode
    rhs_trace: Vec<u32>, // advice: each rhs as input of each opcode
    out_trace: Vec<u32>, // advice: each out as output of each opcode

    // each output is written to stack[top], considered as writing to stack[top]
    // each lhs and rhs correspond to stack[top - 1] and stack[top] respectively, considered as reading from stack[top]
}



