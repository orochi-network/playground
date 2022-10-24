use strum::IntoEnumIterator;
use zkDVM_halo2::{test::{test_executing_gcd::TestExecutingGreatestCommonDivisor, program_execution_handler::ProgramExecutionHandler}, dummy_virtual_machine::{
    opcode::Opcode,
}, utils::numeric_encoding::NumericEncoding};

fn main() {
    TestExecutingGreatestCommonDivisor::execute(100);
}
