use crate::dvm::opcode_definition::OpcodeWithParams;

pub trait ProgramMemoryMaker<const INPUT_SIZE: usize> {
    fn make_program_memory(inputs: [u32; INPUT_SIZE]) -> Vec<OpcodeWithParams>;
}