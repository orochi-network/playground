use crate::dummy_virtual_machine::{
    opcode::{Opcode},
    numeric_encoding::NumericEncoding,
    stack_requirement::StackRequirement,
};

use super::logic::not;

fn compute_next_program_counter(
    current_program_counter: u32, // hidden
    current_stack_depth: u32, // hidden
    read_access_value_1: u32, // hidden // top value of the stack
    read_access_value_2: u32, // hidden // 2nd top value of the stack
    opcode: u32, // public
    program_memory_length: u32, // public constant
    error_index: u32, // public constant
    stop_index: u32, // public constant
) -> u32 {
    // computing is_stack_depth_reasonable
    // notice that opcode is public => opcode.get_stack_depth_minimum() is publicly known
    let opcode = Opcode::from_u32(opcode);
    let is_stack_depth_reasonable = current_stack_depth >= (opcode.get_minimum_stack_depth() as u32); 

    // computing is_program_counter_reasonable_after_executing
    // program_memory_length is considered a fixed constant when conducting proof
    // computation is depending on implementation of trait OpcodeExecutionChecker for opcode
    let is_program_counter_reasonable_after_executing = match opcode {
        Opcode::Stop | Opcode::Return | Opcode::Error => {
            // Stop does not move pc => NoError
            // Return and Error move pc to location of Stop => always NoError
            
            false
        },
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
            // pc is moved next
            current_program_counter + 1 < program_memory_length
        },
        Opcode::Div | Opcode::Mod => {
            let b = read_access_value_2;
            b != 0 && current_program_counter + 1 < program_memory_length
        },
        Opcode::Jump => {
            let destination = read_access_value_1;
            destination < program_memory_length // it jumps to destination, so destination must be valid
        },
        Opcode::Jumpi => {
            let (destination, condition) = (read_access_value_1, read_access_value_2);

            (
                ((condition == 0) as u32) * (program_memory_length + 1) // if condition is zeo then new_pc = pc + 1
                + (1 - (condition == 0) as u32) * destination // else new_pc = destination
            ) < program_memory_length // check new_pc < program_memory_length
        },
    };

    let is_not_error = is_stack_depth_reasonable && is_program_counter_reasonable_after_executing;

    // now output result
    match opcode {
        Opcode::Stop => {
            (not(is_not_error) as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * current_program_counter // else, program counter is unchanged
        },
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
            (not(is_not_error) as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * (current_program_counter + 1) // else, program counter is set to be pc + 1
        },
        Opcode::Return | Opcode::Error => {
            (not(is_not_error) as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * stop_index // else, program counter is set to be stop_index
        },
        Opcode::Jump => {
            let destination = read_access_value_1;
            (not(is_not_error) as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * destination // else, program counter is set to be destination
        },
        Opcode::Jumpi => {
            let (destination, condition) = (read_access_value_1, read_access_value_2);
            (not(is_not_error) as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * ((condition > 0) as u32) * destination // if condition != 0, jump to destination
            + (is_not_error as u32) * (not(condition > 0) as u32) * (current_program_counter + 1) // if condition == 0, jump to pc + 1
        },
    }
}