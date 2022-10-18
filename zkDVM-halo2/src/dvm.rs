use crate::opcode_definition::{Opcode, OpcodeWithParams, StackTopRequirement, ErrorCode, NumericEncoding};

struct DummyVirtualMachine {
    program_memory: Vec<OpcodeWithParams>,
    pc: usize,
    stack: Vec<u32>,
    top: usize,
}

impl DummyVirtualMachine {
    fn new(program_memory: &Vec<OpcodeWithParams>) -> Self {
        let mut new_program_memory = program_memory.clone();
        new_program_memory.push(OpcodeWithParams::new(Opcode::Err, Option::None)); // padding errorcode to jump if necessary
        Self {
            program_memory: new_program_memory,
            pc: 1,
            stack: vec![0; 2],
            top: 1,
        }
    }

    fn is_top_reaching_limit(&self) -> bool {
        if self.top + 1 == self.stack.len() {
            return true;
        }
        false
    }

    fn push_element_to_stack(&self, new_element: &u32) {
        if self.is_top_reaching_limit() {
            self.stack.push(new_element.clone());
        } else {
            self.stack[self.top + 1] = new_element.clone();
        }
        self.top += 1;
    }

    fn get_error_index(&self) -> usize {
        self.program_memory.len() - 1
    }
}

trait Execution {
    fn execute_single_step(&mut self);
}

impl Execution for DummyVirtualMachine {

    fn execute_single_step(&mut self) {

        let mut opcode = self.program_memory[self.pc].get_opcode();
        let mut param = self.program_memory[self.pc].get_param();
        let mut stack = &mut self.stack;
        let mut top = &mut self.top;

        // check top of stack to be true
        if self.top.clone() < opcode.get_stack_top_minimum() {
            self.push_element_to_stack(&ErrorCode::IncorrectStackAccess.to_u32());
            self.pc = self.get_error_index();
            return;
        }

        // then now execute
        match opcode {
            Opcode::Add => {
                stack[top.clone() - 1] = stack[top.clone() - 1] + stack[top.clone()];
                *top -= 1;
            },
            Opcode::Sub => {
                stack[top.clone() - 1] = stack[top.clone() - 1] - stack[top.clone()];
                *top -= 1;
            },
            Opcode::Mul => {
                stack[top.clone() - 1] = stack[top.clone() - 1] * stack[top.clone()];
                *top -= 1;
            },
            Opcode::Div => {
                stack[top.clone() - 1] = stack[top.clone() - 1] / stack[top.clone()];
                *top -= 1;
            },
            Opcode::Push => {
                self.push_element_to_stack(&param);
            },
            Opcode::Pop => {
                *top -= 1;
            },
            Opcode::Ret => {
                // do nothing, top remains
            },
            Opcode::Swap => {
                (stack[top.clone() - 1], stack[top.clone()]) = (stack[top.clone()], stack[top.clone() - 1]);
            },
            Opcode::Jump => {
                if stack[top.clone()] < (self.program_memory.len() as u32) {
                    *pc = stack[top.clone()] as usize;
                } else {
                    
                }
            },
            Opcode::Jumpi => {
                if stack[top.clone() - 1] - 1 != 0 {
                    if stack[top] < (program_memory.len() as u32) {
                        *pc = 
                    }
                }
            }
            _ => {},
        }

        ErrorCode::NoError
    }
}