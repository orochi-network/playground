pub const MAXIMUM_NUM_READS_PER_OPCODE: usize = 4;
pub const MAXIMUM_NUM_WRITES_PER_OPCODE: usize = 2;
pub const MAXIMUM_NUM_ACCESSES_PER_OPCODE: usize = MAXIMUM_NUM_READS_PER_OPCODE + MAXIMUM_NUM_WRITES_PER_OPCODE;
pub const MAXIMUM_NUM_OPCODE_PARAMS_PER_OPCODE: usize = 1;