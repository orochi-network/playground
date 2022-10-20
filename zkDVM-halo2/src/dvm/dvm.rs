use crate::dvm::raw_execution_trace;

use super::opcode_definition::{Opcode, OpcodeWithParams, ErrorCode, StackRequirement};
use super::stack::Stack;
use super::program_memory::ProgramMemory;
use super::raw_execution_trace::RawExecutionTrace;
use super::numeric_encoding::NumericEncoding;
use crate::dvm::direction::Direction;

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
    fn execute_single_step(&mut self, execution_trace: &mut RawExecutionTrace);
    fn execute(&mut self, execution_length: usize) -> (u32, ErrorCode, RawExecutionTrace);
}

impl Execution for DummyVirtualMachine {

    fn execute_single_step(&mut self, execution_trace: &mut RawExecutionTrace) {

        // we assume that at this point, both program_counter and stack.depth are set correctly
        assert!(self.program_memory.is_program_counter_reasonable());

        // get current opcode
        let opcode_with_param = self.program_memory.get_current_opcode_with_params();

        // check where depth of stack is reasonable
        if self.stack.get_depth() < opcode_with_param.get_opcode().get_stack_depth_minimum() {
            self.stack.push(ErrorCode::IncorrectStackAccess.to_u32());
            self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());

            // update execution trace
            execution_trace.push(
                Direction::Error,
                self.program_memory.get_program_counter()
            );
            return;
        } else if !self.program_memory.is_program_counter_reasonable() { // check program pc is reasonable
            self.stack.push(ErrorCode::IncorrectProgramCounter.to_u32());
            self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());
            
            // update execution trace
            execution_trace.push(
                Direction::Error,
                self.program_memory.get_program_counter()
            );
            return;
        } else {
            // then now execute
            // referring here for the use of opcodes https://ethervm.io/

            match opcode_with_param.get_opcode() {
                Opcode::Stop => {
                    // do nothing

                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Add => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a + b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();

                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Sub => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a - b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();

                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Mul => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a * b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();

                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Div => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    if b == 0 {
                        self.stack.push(ErrorCode::DivisionByZero.to_u32());
                        self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());

                        // update execution trace
                        execution_trace.push(
                            Direction::Error,
                            self.program_memory.get_program_counter()
                        );
                    } else {
                        let result = a / b;
                        self.stack.push(result);
                        self.program_memory.next_program_counter();

                        // update execution trace
                        execution_trace.push(
                            Direction::Normal,
                            self.program_memory.get_program_counter()
                        );
                    }
                },
                Opcode::Mod => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    if b == 0 {
                        self.stack.push(ErrorCode::DivisionByZero.to_u32());
                        self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());

                        // update execution trace
                        execution_trace.push(
                            Direction::Error,
                            self.program_memory.get_program_counter()
                        );
                    } else {
                        let result = a % b;
                        self.stack.push(result);
                        self.program_memory.next_program_counter();

                        // update execution trace
                        execution_trace.push(
                            Direction::Normal,
                            self.program_memory.get_program_counter()
                        );
                    }
                },
                Opcode::Push4 => {
                    self.stack.push(opcode_with_param.get_param().unwrap());
                    self.program_memory.next_program_counter();
                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Dup2 => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.stack.push(b);
                    self.stack.push(a);
                    self.stack.push(b);
                    self.program_memory.next_program_counter();
                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Pop => {
                    self.stack.pop();
                    self.program_memory.next_program_counter();
                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Return => {
                    let result = self.stack.pop();
                    self.result = result;
                    self.error_code = ErrorCode::NoError;
                    self.program_memory.next_program_counter_with_destination(self.program_memory.get_stop_index());
                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Swap1 => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.stack.push(a);
                    self.stack.push(b);
                    self.program_memory.next_program_counter();
                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Jump => {
                    let destination = self.stack.pop();
                    self.program_memory.next_program_counter_with_destination(destination as usize);
                    // update execution trace
                    execution_trace.push(
                        Direction::Jump,
                        self.program_memory.get_program_counter()
                    );
                },
                Opcode::Jumpi => {
                    let destination = self.stack.pop();
                    let condition = self.stack.pop();
                    if condition > 0 {
                        self.program_memory.next_program_counter_with_destination(destination as usize);

                        // update execution trace
                        execution_trace.push(
                            Direction::Jump,
                            self.program_memory.get_program_counter()
                        );
                    } else {
                        self.program_memory.next_program_counter();

                        // update execution trace
                        execution_trace.push(
                            Direction::Normal,
                            self.program_memory.get_program_counter()
                        );
                    }
                }, 
                Opcode::Error => {
                    let error_code = self.stack.pop();
                    self.error_code = ErrorCode::from_u32(error_code);
                    self.program_memory.next_program_counter_with_destination(self.program_memory.get_stop_index());

                    // update execution trace
                    execution_trace.push(
                        Direction::Normal,
                        self.program_memory.get_program_counter()
                    );
                },
            }
        }   

        // check pc true
    }

    // execute and return result with corresponding error code (ErrorCode::NoError == 0 if there is no error)
    fn execute(&mut self, execution_length: usize) -> (u32, ErrorCode, RawExecutionTrace) {
        let mut execution_trace = RawExecutionTrace::new(
            self.program_memory.clone(),
            self.get_program_memory().get_program_counter(),
        );
        for _ in 0..execution_length {
            self.execute_single_step(&mut execution_trace);
        }

        // return
        (
            self.result,
            self.error_code.clone(),
            execution_trace,
        )
    }
}