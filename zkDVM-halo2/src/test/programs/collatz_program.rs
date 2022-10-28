use crate::dummy_virtual_machine::{opcode_with_params::OpcodeWithParams, opcode::Opcode};

pub const NUM_COLLATZ_INPUTS: usize = 1;

pub fn make_collatz_program_memory(inputs: &[u32; NUM_COLLATZ_INPUTS]) -> Vec<OpcodeWithParams> {
    let n = inputs[0];
    vec![
        OpcodeWithParams::new(Opcode::Push4, &[Some(n)]), // 0
        // compute n - 1
        OpcodeWithParams::new(Opcode::Push4, &[Some(1)]), // 1.   [1, n
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // 2. [n, 1, n
        OpcodeWithParams::new(Opcode::Sub, &[None]), // 3. [n - 1, n
        // if n - 1 == 0 then jump to return index
        OpcodeWithParams::new(Opcode::Push4, &[Some(26)]), // 4. [return index, n - 1, n
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // 5. [n
        // else if n - 1 != 0 {
        // if n == 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(26)]), // 6. push return index [return index, n
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // 7. [n, return index, n]
        OpcodeWithParams::new(Opcode::Swap1, &[None]), // 8. [return index, n, n]
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // 9. [n
        // else if n != 0: {
        // compute n mod 2
        OpcodeWithParams::new(Opcode::Push4, &[Some(2)]), // 10.[2, n]
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // 11. [n, 2, n]
        OpcodeWithParams::new(Opcode::Mod, &[None]), // 12. [n mod 2, n]
        // deciding jump if n mod 2 == 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(21)]), // 13. [index for handling n mod 2, n mod 2, n]
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // 14. [n]
        // if n mod 2 != 0 then n = 3n + 1
        OpcodeWithParams::new(Opcode::Push4, &[Some(3)]), // 15. [3, n]
        OpcodeWithParams::new(Opcode::Mul, &[None]), // 16. [3n]
        OpcodeWithParams::new(Opcode::Push4, &[Some(1)]), // 17. [1, 3n]
        OpcodeWithParams::new(Opcode::Add, &[None]), // 18. [3n + 1]
        OpcodeWithParams::new(Opcode::Push4, &[Some(24)]), // 19. [1, 3n + 1]
        OpcodeWithParams::new(Opcode::Jump, &[None]), // 20. pc jump to 24, [3n + 1]
        // if n mod 2 == 0 then n = n / 2
        OpcodeWithParams::new(Opcode::Push4, &[Some(2)]), // 21. [2, n]
        OpcodeWithParams::new(Opcode::Swap1, &[None]), // 22. [n, 2]
        OpcodeWithParams::new(Opcode::Div, &[None]), // 23. [n / 2]

        // jump back
        OpcodeWithParams::new(Opcode::Push4, &[Some(1)]), // 24. [1, n after n/2 or 3n + 1]
        OpcodeWithParams::new(Opcode::Jump, &[None]), // 25. pc jump to 1, [n after n/2 or 3n+1]

        // return here
        OpcodeWithParams::new(Opcode::Return, &[None]), //  26. return
    ]
}