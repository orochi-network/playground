use crate::runtime::opcode_util::opcode::Opcode;
use crate::runtime::opcode_util::opcode_with_immediate_value::{OpcodeWithImmediateValue};

pub const NUM_GCD_INPUTS: usize = 2;

pub fn make_gcd_program_memory(inputs: &[u32; NUM_GCD_INPUTS]) -> Vec<OpcodeWithImmediateValue> {
    let a = inputs[0];
    let b = inputs[1];
    vec![
        OpcodeWithImmediateValue::new(Opcode::Push4, Some(a)), // index 0
        OpcodeWithImmediateValue::new(Opcode::Push4, Some(b)), // index 1
        // stack[b, a]


        // start the loop here
        
        // first duplicate
        OpcodeWithImmediateValue::new(Opcode::Dup2, None), // index 2
        OpcodeWithImmediateValue::new(Opcode::Dup2, None), // index 3 // stack: [b, a, b, a]

        // now swap
        OpcodeWithImmediateValue::new(Opcode::Swap1, None), // index 4
        // stack: [a, b, b, a]

        // then compure r = a mod b
        OpcodeWithImmediateValue::new(Opcode::Mod, None), // index 5 // stack[r, b, a]
        
        // by assigning a = b, b = r => view stack as [b, a, _]

        // now testing whether b == 0
        OpcodeWithImmediateValue::new(Opcode::Push4, Some(2)), // index 6 // push destination == 2 to the stack // stack_utils[2, b, a]
        
        // duplicate
        OpcodeWithImmediateValue::new(Opcode::Dup2, None), // index 7 // stack[b, 2, b, a]

        // then swap
        OpcodeWithImmediateValue::new(Opcode::Swap1, None), // index 8 // stack[2, b, b, a]

        // then jumpi
        OpcodeWithImmediateValue::new(Opcode::Jumpi, None), // index 9 // if b != 0 then jump to destination 2, else pc += 1

        // stack[b, a]
        // now return by dirst duplicating
        OpcodeWithImmediateValue::new(Opcode::Dup2, None), // index 10 // stack[a, b, a]
        OpcodeWithImmediateValue::new(Opcode::Return, None), // index 11 // return a :@)

    ]
}