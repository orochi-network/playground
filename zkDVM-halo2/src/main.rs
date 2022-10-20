use zkDVM_halo2::test::{self, test_opcode_definition::TestOpcodeDefinition, test_executing_gcd::TestExecutingGreatestCommonDivisor, program_execution_handler::ProgramExecutionHandler};

fn main() {
    TestExecutingGreatestCommonDivisor::execute(40);
}
