use crate::runtime::opcode_util::opcode::Opcode;
use crate::runtime::opcode_util::opcode_with_params::OpcodeWithParams;

pub const NUM_COLLATZ_INPUTS: usize = 1;

pub fn make_collatz_program_memory(inputs: &[u32; NUM_COLLATZ_INPUTS]) -> Vec<OpcodeWithParams> {
    let n = inputs[0];
    vec![
        OpcodeWithParams::new(Opcode::Push4, &[Some(n)]), // 0. [n]
        // compute n - 1
        OpcodeWithParams::new(Opcode::Push4, &[Some(1)]), // 1.   [1, n
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // 2. [n, 1, n
        OpcodeWithParams::new(Opcode::Sub, &[None]), // 3. [n - 1, n
        OpcodeWithParams::new(Opcode::Push4, &[Some(7)]), // 4. [index for n - 1 != 0, n - 1, n
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // 5. [n

        // handling n - 1 == 0
        OpcodeWithParams::new(Opcode::Return, &[None]), // 6. return n if n - 1 == 0

        // handling n - 1 != 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(13)]), // 7. [index for n != 0 and n != 1, n
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // 8. [n, index for n != 0 and n != 1, n]
        OpcodeWithParams::new(Opcode::Swap1, &[None]), // 9. [index for n != 0 and n != 1, n, n]
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // 10. [n

        // handling n - 1 != 0 and n == 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(29)]), // 11. [return index, n
        OpcodeWithParams::new(Opcode::Jump, &[None]), // 12. [n

        // handling n - 1 != 0 and n != 0
        // compute n mod 2
        OpcodeWithParams::new(Opcode::Push4, &[Some(2)]), // 13. [2, n]
        OpcodeWithParams::new(Opcode::Dup2, &[None]), // 14. [n, 2, n]
        OpcodeWithParams::new(Opcode::Mod, &[None]), // 15. [n mod 2, n]
        OpcodeWithParams::new(Opcode::Push4, &[Some(23)]), // 16. [index for n mod 2 != 0, n mod 2, n]
        OpcodeWithParams::new(Opcode::Jumpi, &[None]), // 17. [n

        // handling n mod 2 == 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(2)]), // 18. [2, n]
        OpcodeWithParams::new(Opcode::Swap1, &[None]), // 19. [n, 2]
        OpcodeWithParams::new(Opcode::Div, &[None]), // 20. [n / 2]
        OpcodeWithParams::new(Opcode::Push4, &[Some(27)]), // 21. [27, n / 2]
        OpcodeWithParams::new(Opcode::Jump, &[None]), // 22. [n / 2]

        // handling n mod 2 != 0
        OpcodeWithParams::new(Opcode::Push4, &[Some(3)]), // 23. [3, n]
        OpcodeWithParams::new(Opcode::Mul, &[None]), // 24. [3n]
        OpcodeWithParams::new(Opcode::Push4, &[Some(1)]), // 25. [1, 3n]
        OpcodeWithParams::new(Opcode::Add, &[None]), // 26. [3n + 1]
        


        OpcodeWithParams::new(Opcode::Push4, &[Some(1)]), // 27. [index after handling n/2 or 3n + 1, ...]
        OpcodeWithParams::new(Opcode::Jump, &[None]), // 28. 

        // return here
        OpcodeWithParams::new(Opcode::Return, &[None]), //  29. return
    ]
}