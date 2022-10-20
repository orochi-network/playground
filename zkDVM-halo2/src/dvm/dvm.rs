use super::opcode_definition::{Opcode, OpcodeWithParams, ErrorCode, NumericEncoding, StackRequirement};
use super::stack::Stack;
use super::program_memory::ProgramMemory;
use crate::proof::raw_execution_trace::RawExecutionTrace;

pub struct DummyVirtualMachine {
    program_memory: ProgramMemory,
    stack: Stack,
    result: u32,
    error_code: ErrorCode,
}

impl DummyVirtualMachine {
    pub fn new(program_memory: &Vec<OpcodeWithParams>) -> Self {
        let mut new_program_memory = program_memory.clone();
        Self {
            program_memory: ProgramMemory::new(new_program_memory),
            stack: Stack::new(),
            result: 0,
            error_code: ErrorCode::NoReturn,
        }
    }

    pub fn get_program_memory(&self) -> &ProgramMemory {
        &self.program_memory
    }

    pub fn get_result(&self) -> u32 {
        self.result
    }

    pub fn get_error_code(&self) -> ErrorCode {
        self.error_code.clone()
    }
}

pub trait Execution {
    fn execute_single_step(&mut self, wrapped_execution_trace: &Option<&mut RawExecutionTrace>);
    fn execute(&mut self, execution_length: usize, wrapped_execution_trace: &Option<&mut RawExecutionTrace>) -> (u32, ErrorCode);
}

impl Execution for DummyVirtualMachine {

    fn execute_single_step(&mut self, wrapped_execution_trace: &Option<&mut RawExecutionTrace>) {

        // we assume that at this point, both program_counter and stack.depth are set correctly
        assert!(self.program_memory.is_program_counter_reasonable());

        // get current opcode
        let opcode_with_param = self.program_memory.get_current_opcode_with_params();

        // check where depth of stack is reasonable
        if self.stack.get_depth() < opcode_with_param.get_opcode().get_stack_depth_minimum() {
            self.stack.push(ErrorCode::IncorrectStackAccess.to_u32());
            self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());
            return;
        }

        // check program pc is reasonable
        if !self.program_memory.is_program_counter_reasonable() {
            self.stack.push(ErrorCode::IncorrectProgramCounter.to_u32());
            self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());
            return;
        }

        // then now execute
        // referring here for the use of opcodes https://ethervm.io/
        match opcode_with_param.get_opcode() {
            Opcode::Stop => {
                // do nothing
            },
            Opcode::Add => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                let result = a + b;
                self.stack.push(result);
                self.program_memory.next_program_counter();
            },
            Opcode::Sub => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                let result = a - b;
                self.stack.push(result);
                self.program_memory.next_program_counter();
            },
            Opcode::Mul => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                let result = a * b;
                self.stack.push(result);
                self.program_memory.next_program_counter();
            },
            Opcode::Div => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                if b == 0 {
                    self.stack.push(ErrorCode::DivisionByZero.to_u32());
                    self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());
                } else {
                    let result = a / b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();
                }
            },
            Opcode::Mod => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                if b == 0 {
                    self.stack.push(ErrorCode::DivisionByZero.to_u32());
                    self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());
                } else {
                    let result = a % b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();
                }
            },
            Opcode::Push4 => {
                self.stack.push(opcode_with_param.get_param().unwrap());
                self.program_memory.next_program_counter();
            },
            Opcode::Dup2 => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
                self.program_memory.next_program_counter();
            },
            Opcode::Pop => {
                self.stack.pop();
                self.program_memory.next_program_counter();
            },
            Opcode::Return => {
                let result = self.stack.pop();
                self.result = result;
                self.error_code = ErrorCode::NoError;
                self.program_memory.next_program_counter_with_destination(self.program_memory.get_stop_index());
            },
            Opcode::Swap1 => {
                let a = self.stack.pop();
                let b = self.stack.pop();
                self.stack.push(a);
                self.stack.push(b);
                self.program_memory.next_program_counter();
            },
            Opcode::Jump => {
                let destination = self.stack.pop();
                self.program_memory.next_program_counter_with_destination(destination as usize);
            },
            Opcode::Jumpi => {
                let destination = self.stack.pop();
                let condition = self.stack.pop();
                if condition > 0 {
                    self.program_memory.next_program_counter_with_destination(destination as usize);
                } else {
                    self.program_memory.next_program_counter();
                }
            }, 
            Opcode::Error => {
                let error_code = self.stack.pop();
                self.error_code = ErrorCode::from_u32(error_code);
                self.program_memory.next_program_counter_with_destination(self.program_memory.get_stop_index());
            },
        }

        // check pc true
    }

    // execute and return result with corresponding error code (ErrorCode::NoError == 0 if there is no error)
    fn execute(&mut self, execution_length: usize, wrapped_execution_trace: &Option<&mut RawExecutionTrace>) -> (u32, ErrorCode) {
        for _ in 0..execution_length {
            self.execute_single_step(wrapped_execution_trace);
        }
        (
            self.result,
            self.error_code.clone(),
        )
    }
}