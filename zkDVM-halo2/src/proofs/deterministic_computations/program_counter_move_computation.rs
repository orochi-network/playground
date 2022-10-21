use crate::dvm::{
    opcode_definition::{Opcode, StackRequirement},
    numeric_encoding::NumericEncoding,
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
    let is_stack_depth_reasonable = current_stack_depth >= (opcode.get_stack_depth_minimum() as u32); 

    // computing is_program_counter_reasonable
    // program_memory_length is considered a fixed constant when conducting proof
    let is_program_counter_reasonable = current_program_counter < program_memory_length;

    match opcode {
        Opcode::Stop => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * current_program_counter
        },
        Opcode::Add => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Sub => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Mul => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Div => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        }, 
        Opcode::Mod => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Push4 => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Dup2 => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Pop => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Return => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * stop_index
        },
        Opcode::Swap1 => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (current_program_counter + 1)
        },
        Opcode::Jump => {
            let destination = &read_access_value_1;
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * destination
        },
        Opcode::Jumpi => {
            let (destination, condition) = &(read_access_value_1, read_access_value_2);
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * ((*condition > 0) as u32) * destination
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * (not(*condition > 0) as u32) * destination
        },
        Opcode::Error => {
            (not(is_stack_depth_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (not(is_program_counter_reasonable) as u32) * error_index
            + (is_stack_depth_reasonable as u32) * (is_program_counter_reasonable as u32) * stop_index
        }
    }
}