// Context of Dummy Virtual Machine
pub struct DVMContext {
    stack: Vec<i32>,
    popped: i32,
    result: i32,
    terminated: bool,
}

// Operation Code in binary form
#[derive(Copy, Clone, PartialEq, Debug)]
// DVM's opcode
pub enum BinaryCode {
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Push = 0x05,
    Pop = 0x06,
    Ret = 0x07,
    Swap = 0x08,
    Stop = 0xfe,
    Invalid = 0xff,
}

impl BinaryCode {
    // Execute the opcode with the Dummy Virtual Machine's context
    pub fn exec(&self, ctx: &mut DVMContext, param: i32) {
        match *self {
            Self::Add => {
                if ctx.stack.len() < 2 {
                    panic!("Can not perform ADD, stack deep is {}", ctx.stack.len());
                }
                let a = ctx.stack.pop().unwrap();
                let b = ctx.stack.pop().unwrap();
                ctx.stack.push(a + b);
                println!("ADD\t(${:#08x} + ${:#08x})", a, b);
            }
            Self::Sub => {
                if ctx.stack.len() < 2 {
                    panic!("Can not perform SUB, stack deep is {}", ctx.stack.len());
                }
                let a = ctx.stack.pop().unwrap();
                let b = ctx.stack.pop().unwrap();
                ctx.stack.push(a - b);
                println!("SUB\t(${:#08x} - ${:#08x})", a, b);
            }
            Self::Mul => {
                if ctx.stack.len() < 2 {
                    panic!("Can not perform MUL, stack deep is {}", ctx.stack.len());
                }
                let a = ctx.stack.pop().unwrap();
                let b = ctx.stack.pop().unwrap();
                ctx.stack.push(a * b);

                println!("MUL\t(${:#08x} * ${:#08x})", a, b);
            }
            Self::Div => {
                if ctx.stack.len() < 2 {
                    panic!("Can not perform ADD, stack deep is {}", ctx.stack.len());
                }
                let a = ctx.stack.pop().unwrap();
                let b = ctx.stack.pop().unwrap();
                if b == 0 {
                    panic!("Divide by 0");
                }
                ctx.stack.push(a / b);
                println!("DIV\t(${:#08x} / ${:#08x})", a, b);
            }
            Self::Push => {
                println!("PUSH\t${:#08x}", param);
                ctx.stack.push(param);
            }
            Self::Pop => {
                ctx.popped = ctx.stack.pop().unwrap();
                println!("POP");
            }
            Self::Ret => {
                ctx.result = ctx.stack.pop().unwrap();
                ctx.terminated = true;
                println!("RET\t${:#08x}", ctx.result);
            }
            Self::Swap => {
                let a = ctx.stack.pop().unwrap();
                let b = ctx.stack.pop().unwrap();
                ctx.stack.push(a);
                ctx.stack.push(b);
                println!("SWAP\t${:#08x} <-> {:#08x}", a, b);
            }
            Self::Stop => {
                ctx.terminated = true;
                println!("STOP");
            }
            Self::Invalid => panic!("Hello darkness, my old friend!"),
        }
        println!("\t\t\t\t\t{:?}", ctx.stack);
    }

    pub fn from(bin: u8) -> BinaryCode {
        match bin {
            0x01 => Self::Add,
            0x02 => Self::Sub,
            0x03 => Self::Mul,
            0x04 => Self::Div,
            0x05 => Self::Push,
            0x06 => Self::Pop,
            0x07 => Self::Ret,
            0x08 => Self::Swap,
            0xfe => Self::Stop,
            _ => Self::Invalid,
        }
    }
}

// Opcode is the combine of BinaryCode and parameters
#[derive(Debug)]
pub struct Opcode(BinaryCode, i32);

impl Opcode {
    pub fn new(bin_code: BinaryCode, param: i32) -> Self {
        Opcode(bin_code, param)
    }
    pub fn exec(&self, ctx: &mut DVMContext) {
        self.0.exec(ctx, self.1)
    }
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
