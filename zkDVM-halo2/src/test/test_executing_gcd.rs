use crate::{test::program_memory_maker::ProgramMemoryMaker, dvm::{
        opcode_definition::{
            OpcodeWithParams, Opcode
        }, 
        dvm::{
            DummyVirtualMachine, Execution
        }, direction
    }
};

use super::{
    program_execution_handler::ProgramExecutionHandler,
};

pub struct TestExecutingGreatestCommonDivisor {

}

impl TestExecutingGreatestCommonDivisor {
    fn raw_gcd(a: u32, b: u32) -> u32{ // assuming b != 0 to test the error at the beginning
        let mut new_a = a;
        let mut new_b = b;
        loop {
            let r = new_a % new_b;
            new_a = new_b;
            new_b = r;
            if new_b == 0 {
                break;
            }
        };
        new_a
    }
}

impl ProgramMemoryMaker<2> for TestExecutingGreatestCommonDivisor {
    fn make_program_memory(inputs: [u32; 2]) -> Vec<OpcodeWithParams> {
        let a = inputs[0];
        let b = inputs[1];
        vec![
            OpcodeWithParams::new(Opcode::Push4, Some(a)), // index 0
            OpcodeWithParams::new(Opcode::Push4, Some(b)), // index 1
            // stack[b, a]


            // start the loop here
            
            // first duplicate
            OpcodeWithParams::new(Opcode::Dup2, None), // index 2
            OpcodeWithParams::new(Opcode::Dup2, None), // index 3 // stack: [b, a, b, a]

            // now swap
            OpcodeWithParams::new(Opcode::Swap1, None), // index 4
            // stack: [a, b, b, a]

            // then compure r = a mod b
            OpcodeWithParams::new(Opcode::Mod, None), // index 5 // stack[r, b, a] 
            
            // by assigning a = b, b = r => view stack as [b, a, _]

            // now testing whether b == 0
            OpcodeWithParams::new(Opcode::Push4, Some(2)), // index 6 // push destination == 2 to the stack // stack[2, b, a]
            
            // duplicate
            OpcodeWithParams::new(Opcode::Dup2, None), // index 7 // stack[b, 2, b, a]

            // then swap
            OpcodeWithParams::new(Opcode::Swap1, None), // index 8 // stack[2, b, b, a]

            // then jumpi
            OpcodeWithParams::new(Opcode::Jumpi, None), // index 9 // if b != 0 then jump to destination 2, else pc += 1

            // stack[b, a]
            // now return by dirst duplicating
            OpcodeWithParams::new(Opcode::Dup2, None), // index 10 // stack[a, b, a]
            OpcodeWithParams::new(Opcode::Return, None), // index 11 // return a :@)

        ]
    }
}

impl ProgramExecutionHandler<2> for TestExecutingGreatestCommonDivisor {
    fn execute(num_steps: usize) {
        let test_vector = [
            [0, 0],
            [10, 0], 
            [0, 4],
            [4, 12],
            [20, 100],
            [15, 7],
            [324, 2442],
        ];
        for input in test_vector {
            let mut dummy_vm = DummyVirtualMachine::new(
                &Self::make_program_memory(input)
            );
    
            dummy_vm.get_program_memory().display();
    
            let (result, error_code, execution_trace) = dummy_vm.execute(num_steps);
            println!("Input = {:?}, Result = {}, Error Code = {:?}", input, result, error_code);
            
            let direction_trace = execution_trace.get_direction_trace();
            println!("Directions ({} elements): ", direction_trace.len());
            print!("[");
            for direction in direction_trace {
                print!("{:?},", direction);
            }
            println!("]");

            let program_counter_trace = execution_trace.get_program_counter_trace();
            println!("Program counters ({} elements): ", program_counter_trace.len());
            print!("[");
            for program_counter in program_counter_trace {
                print!("{:?},", program_counter);
            }
            println!("]");
        }
    }
}