use super::opcode_definition::{Opcode, OpcodeWithParams, ErrorCode};
use super::stack::Stack;
use super::program_memory::ProgramMemory;
use super::raw_execution_trace::RawExecutionTrace;
use super::numeric_encoding::NumericEncoding;
use super::execution_checker::ExecutionChecker;
use crate::dvm::direction::Direction;
use crate::dvm::opcode_definition::StackRequirement;

pub struct DummyVirtualMachine {
    program_memory: ProgramMemory,
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
            stack: Stack::new(),
            result: 0,
            error_code: ErrorCode::NoReturn,
            time_tag: 0,
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
        let opcode_with_param = self.get_program_memory().get_current_opcode_with_params();

        // first record the necessary read values
        let read_access_value_1 = self.stack[self.stack.get_depth() - 1];
        let read_access_value_2 = self.stack[self.stack.get_depth() - 2];
        let depth_before_changed = self.stack.get_depth();
        let direction: Direction;

        // check where depth of stack is reasonable
        if self.stack.get_depth() < opcode_with_param.get_opcode().get_stack_depth_minimum() {
            self.stack.push(ErrorCode::IncorrectStackAccess.to_u32());
            self.program_memory.set_program_counter(self.program_memory.get_error_index());

            direction = Direction::Error;
        } else {

            // check error code before executing
            let error_code = opcode_with_param.get_opcode().get_error_after_executing(
                read_access_value_1, read_access_value_2, 
                &self.program_memory
            );

            // then now execute
            // referring here for the use of opcodes https://ethervm.io/
            match opcode_with_param.get_opcode() {
                Opcode::Stop => {
                    // do nothing

                    direction = Direction::Normal;
                },
                Opcode::Add => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a + b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();

                    direction = Direction::Normal;
                },
                Opcode::Sub => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a - b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();

                    direction = Direction::Normal;
                },
                Opcode::Mul => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let result = a * b;
                    self.stack.push(result);
                    self.program_memory.next_program_counter();

                    direction = Direction::Normal;
                },
                Opcode::Div => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    if b == 0 {
                        self.stack.push(ErrorCode::DivisionByZero.to_u32());
                        self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());

                        direction = Direction::Error;
                    } else {
                        let result = a / b;
                        self.stack.push(result);
                        self.program_memory.next_program_counter();

                        direction = Direction::Normal;
                    }
                },
                Opcode::Mod => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    if b == 0 {
                        self.stack.push(ErrorCode::DivisionByZero.to_u32());
                        self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());

                        direction = Direction::Error;
                    } else {
                        let result = a % b;
                        self.stack.push(result);
                        self.program_memory.next_program_counter();

                        direction = Direction::Normal;
                    }
                },
                Opcode::Push4 => {
                    self.stack.push(opcode_with_param.get_param().unwrap());
                    self.program_memory.next_program_counter();
                    
                    direction = Direction::Normal;
                },
                Opcode::Dup2 => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.stack.push(b);
                    self.stack.push(a);
                    self.stack.push(b);
                    self.program_memory.next_program_counter();
                    
                    direction = Direction::Normal;
                },
                Opcode::Pop => {
                    self.stack.pop();
                    self.program_memory.next_program_counter();
                    
                    direction = Direction::Normal;
                },
                Opcode::Return => {
                    let result = self.stack.pop();
                    self.result = result;
                    self.error_code = ErrorCode::NoError;
                    self.program_memory.next_program_counter_with_destination(self.program_memory.get_stop_index());
                    
                    direction = Direction::Normal;
                },
                Opcode::Swap1 => {
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    self.stack.push(a);
                    self.stack.push(b);
                    self.program_memory.next_program_counter();
                    
                    direction = Direction::Normal;
                },
                Opcode::Jump => {
                    let destination = self.stack.pop();
                    self.program_memory.next_program_counter_with_destination(destination as usize);
                    
                    direction = Direction::Jump;
                },
                Opcode::Jumpi => {
                    let destination = self.stack.pop();
                    let condition = self.stack.pop();
                    if condition > 0 {
                        self.program_memory.next_program_counter_with_destination(destination as usize);

                        direction = Direction::Jump;
                    } else {
                        self.program_memory.next_program_counter();

                        direction = Direction::Normal;
                    }
                }, 
                Opcode::Error => {
                    let error_code = self.stack.pop();
                    self.error_code = ErrorCode::from_u32(error_code);
                    self.program_memory.next_program_counter_with_destination(self.program_memory.get_stop_index());

                    direction = Direction::Normal;
                },
            }

            if !self.program_memory.is_program_counter_reasonable() { // check program pc is reasonable
                self.stack.push(ErrorCode::IncorrectProgramCounter.to_u32());
                self.program_memory.next_program_counter_with_destination(self.program_memory.get_error_index());
                
                direction = Direction::Error;
            }
        }   

        execution_trace.push(
            direction, 
            self.get_program_memory().get_program_counter(), 
            &mut self.time_tag, 
            depth_before_changed, 
            read_access_value_1, read_access_value_2, 
            self.stack.get_depth(), 
            self.stack.get_top()
        );

        // check pc true
    }

    // execute and return result with corresponding error code (ErrorCode::NoError == 0 if there is no error)
    fn execute(&mut self, execution_length: usize) -> (u32, ErrorCode, RawExecutionTrace) {
        let mut execution_trace = RawExecutionTrace::new(
            &self.program_memory,
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