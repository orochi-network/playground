use super::opcode::{BinaryCode, Opcode};

// Context of Dummy Virtual Machine
pub struct DVMContext {
    pub stack: Vec<i32>,
    pub popped: i32,
    pub result: i32,
    pub terminated: bool,
}

// Dummy Virtual Machine
pub struct DVM {
    context: DVMContext,
}

impl DVM {
    // Create new instance of DVM with default context
    pub fn new() -> Self {
        DVM {
            context: DVMContext {
                stack: Vec::<i32>::new(),
                popped: 0,
                result: 0,
                terminated: false,
            },
        }
    }

    // Process a given program with DVM
    pub fn process(&mut self, program: Vec<u8>) -> i32 {
        let mut program_ptr = 0;
        while program_ptr < program.len() {
            let bin_code = BinaryCode::from(program[program_ptr]);
            match bin_code {
                BinaryCode::Push => {
                    program_ptr += 1;
                    let param = i32::from_be_bytes(
                        program.as_slice()[program_ptr..program_ptr + 4]
                            .try_into()
                            .unwrap(),
                    );
                    program_ptr += 4;
                    Opcode::new(bin_code, param).exec(&mut self.context)
                }
                _ => {
                    program_ptr += 1;
                    Opcode::new(bin_code, 0).exec(&mut self.context)
                }
            };
        }
        self.context.result
    }
}
