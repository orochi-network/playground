use crate::dvm::dvm::DummyVirtualMachine;

use super::program_memory_maker::ProgramMemoryMaker;

pub trait ProgramExecutionHandler<const INPUT_SIZE: usize>: ProgramMemoryMaker<INPUT_SIZE> {
    fn execute(num_steps: usize);
}