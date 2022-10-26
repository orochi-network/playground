use crate::utils::numeric_encoding::NumericEncoding;

use super::error_code::ErrorCode;
use super::execution::Execution;
use super::opcode::{Opcode};
use super::opcode_with_params::OpcodeWithParams;
use super::stack::Stack;
use super::program_memory::ProgramMemory;
use super::raw_execution_trace::RawExecutionTrace;
use super::opcode_execution_checker::OpcodeExecutionChecker;
use super::stack_requirement::StackRequirement;

pub struct DummyVirtualMachine {
    program_memory: ProgramMemory,
    program_counter: usize,
    stack: Stack,
    result: u32,
    error_code: ErrorCode,
    time_tag: u32,
}

impl DummyVirtualMachine {
    pub fn new(program_memory: &Vec<OpcodeWithParams>) -> Self {
        let mut new_program_memory = program_memory.clone();
        Self {
            program_memory: ProgramMemory::new(new_program_memory),
            program_counter: 0,
            stack: Stack::new(),
            result: 0,
            error_code: ErrorCode::NoReturn,
            time_tag: Stack::NUM_INACCESSIBLE_ELEMENTS as u32,
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

    // add new value to stack and change program counter
    fn update_stack_and_program_counter(&mut self, value: u32, new_program_counter: usize) {
        self.stack.push(value);
        self.program_counter = new_program_counter;
    }

    fn execute_single_step(&mut self, execution_trace: &mut RawExecutionTrace) {

        // we assume that at this point, both program_counter and stack.depth are set correctly
        assert!(self.program_memory.is_program_counter_reasonable(self.program_counter));

        // get current opcode
        let opcode_with_param = self.program_memory[self.program_counter].clone();

        // first record the necessary read values
        let read_stack_value_1 = self.stack[self.stack.get_depth() - 1];
        let read_stack_value_2 = self.stack[self.stack.get_depth() - 2];
        let depth_before_changed = self.stack.get_depth();

        // check where depth of stack is reasonable
        let error_code: ErrorCode;
        if self.stack.get_depth() < opcode_with_param.get_opcode().get_minimum_stack_depth() {
            error_code = ErrorCode::IncorrectStackAccess;
            self.program_counter = self.program_memory.get_error_index();
        } else {

            // check possible error code before executing
            error_code = opcode_with_param.get_opcode().get_error_after_executing(
                read_stack_value_1, read_stack_value_2, 
                &self.program_memory,
                self.program_counter,
            );
        }   

        // then now execute
        // referring here for the use of opcodes https://ethervm.io/
        if error_code == ErrorCode::NoError {
            match opcode_with_param.get_opcode() {
                Opcode::Stop => {
                    // do nothing
                },
                Opcode::Add => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a + b;
                    self.update_stack_and_program_counter(
                        result, 
                        self.program_counter + 1
                    );
                },
                Opcode::Sub => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a - b;
                    self.update_stack_and_program_counter(
                        result, 
                        self.program_counter + 1
                    );
                },
                Opcode::Mul => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a * b;
                    self.update_stack_and_program_counter(
                        result, 
                        self.program_counter + 1
                    );
                },
                Opcode::Div => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a / b;
                    self.update_stack_and_program_counter(
                        result, 
                        self.program_counter + 1
                    );
                },
                Opcode::Mod => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a % b;
                    self.update_stack_and_program_counter(
                        result, 
                        self.program_counter + 1
                    );
                },
                Opcode::Push4 => {
                    self.update_stack_and_program_counter(
                        opcode_with_param.get_param().unwrap(), 
                        self.program_counter + 1
                    );
                },
                Opcode::Dup2 => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.update_stack_and_program_counter(
                        b, 
                        self.program_counter
                    );
                    self.update_stack_and_program_counter(
                        a, 
                        self.program_counter
                    );
                    self.update_stack_and_program_counter(
                        b, 
                        self.program_counter + 1
                    );
                },
                Opcode::Pop => {
                    self.stack.pop();
                    self.program_counter += 1;
                },
                Opcode::Return => {
                    let result = self.stack.pop();
                    self.result = result;
                    self.error_code = ErrorCode::NoError;
                    self.program_counter = self.program_memory.get_stop_index();
                },
                Opcode::Swap1 => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.update_stack_and_program_counter(
                        a, 
                        self.program_counter
                    );
                    self.update_stack_and_program_counter(
                        b, 
                        self.program_counter + 1
                    );
                },
                Opcode::Jump => {
                    let destination = self.stack.pop();
                    self.program_counter = destination as usize;
                },
                Opcode::Jumpi => {
                    let destination = self.stack.pop();
                    let condition = self.stack.pop();
                    if condition > 0 {
                        self.program_counter = destination as usize;
                    } else {
                        self.program_counter += 1;
                    }
                }, 
                Opcode::Error => {
                    let error_code = self.stack.pop();
                    self.error_code = ErrorCode::from_u32(error_code);
                    self.program_counter = self.program_memory.get_stop_index();
                },
            };
        } else {
            self.update_stack_and_program_counter(error_code.to_u32(), self.program_memory.get_error_index());
        }

        execution_trace.push(
            self.program_counter, 
            &mut self.time_tag, 
            depth_before_changed, 
            read_stack_value_1, read_stack_value_2, 
            self.stack.get_depth(), 
            self.stack.get_top(),
            self.stack[self.stack.get_depth() - 2],
            opcode_with_param.get_opcode(),
        );

        // check pc true
    }
}

impl Execution for DummyVirtualMachine {

    // execute and return result with corresponding error code (ErrorCode::NoError == 0 if there is no error)
    fn execute(&mut self, execution_length: usize) -> (u32, ErrorCode, RawExecutionTrace) {
        let mut execution_trace = RawExecutionTrace::new(
            &self.program_memory,
            self.program_counter,
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