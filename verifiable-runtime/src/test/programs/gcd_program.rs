use crate::{runtime::opcode_with_params::OpcodeWithParams, opcode::Opcode};

pub const NUM_GCD_INPUTS: usize = 2;

pub fn make_gcd_program_memory(inputs: &[u32; NUM_GCD_INPUTS]) -> Vec<OpcodeWithParams> {
    let a = inputs[0];
    let b = inputs[1];
    vec![
        OpcodeWithParams::new(Opcode::Push4, &[Some(a)]), // index 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(b)]), // index 1
        // stack[b, a]


        // start the loop here
        
        // first duplicate
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // index 2
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // index 3 // stack: [b, a, b, a]

        // now swap
        OpcodeWithParams::new(Opcode::Swap1, &[None]), // index 4
        // stack: [a, b, b, a]

        // then compure r = a mod b
        OpcodeWithParams::new(Opcode::Mod, &[None]), // index 5 // stack[r, b, a] 
        
        // by assigning a = b, b = r => view stack as [b, a, _]

        // now testing whether b == 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(2)]), // index 6 // push destination == 2 to the stack // stack[2, b, a]
        
        // duplicate
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // index 7 // stack[b, 2, b, a]

        // then swap
        OpcodeWithParams::new(Opcode::Swap1, &[None]), // index 8 // stack[2, b, b, a]

        // then jumpi
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // index 9 // if b != 0 then jump to destination 2, else pc += 1

        // stack[b, a]
        // now return by dirst duplicating
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // index 10 // stack[a, b, a]
        OpcodeWithParams::new(Opcode::Return, &[None]), // index 11 // return a :@)

    ]
}