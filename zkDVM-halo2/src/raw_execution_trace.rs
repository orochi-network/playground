struct OpcodeStructure {
    id: u32,
    first_param: u32,
    second_param: u32,
}

struct RawExecutionTrace {
    program_memory: Vec<OpcodeStructure>, // public: store the sequence of opcodes (encoded into u32)
    pc_trace: Vec<u32>, // advice: store pc after each execution
    opcode_trace: Vec<u32>, // advice: store the encoded opcodes (u32) according to pc_trace
    top_trace: Vec<u32>, // advice: store the top pointer of the stack
    lhs_trace: Vec<u32>, // advice: each lhs as input of each opcode
    rhs_trace: Vec<u32>, // advice: each rhs as input of each opcode
    out_trace: Vec<u32>, // advice: each out as output of each opcode

    // each output is written to stack[top], considered as writing to stack[top]
    // each lhs and rhs correspond to stack[top - 1] and stack[top] respectively, considered as reading from stack[top]
}

// Opcodes encoding
// PUSH (encoded 0): top is increased by 1, no constraint to lhs and rhs, pc += 1
// ADD (encoded 1): top is decreased by 1 with written output = lhs + rhs, pc += 1
// SUB (encoded 2): top is decreased by 1 with written output = lhs - rhs, pc += 1
// MUL (encoded 3): top is decreased by 1 with written output = lhs * rhs, pc += 1
// DIV (encoded 4): top is decreased by 1 with written output = lhs // rhs, pc += 1
// RET (encoded 5): top is kept unchanged with written output = rhs, pc += 1
// CMP new_pc (encoded 6): top is kept unchanged with written output = rhs, pc = new_pc according to 
// CMPI dest new_pc (encoded 7): top is kept unchanged with written output = rhs, pc = (bool(rhs)) * (pc + 1) + (1 - bool(rhs)) * new_pc

