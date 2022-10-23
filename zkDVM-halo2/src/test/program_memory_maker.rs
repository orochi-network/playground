use crate::dummy_virtual_machine::opcode_with_params::OpcodeWithParams;

pub trait ProgramMemoryMaker<const INPUT_SIZE: usize> {
    fn make_program_memory(inputs: [u32; INPUT_SIZE]) -> Vec<OpcodeWithParams>;
}