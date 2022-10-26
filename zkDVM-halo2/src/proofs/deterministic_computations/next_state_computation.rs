use crate::{dummy_virtual_machine::{
    opcode::{Opcode},
    stack_requirement::StackRequirement,
}, utils::numeric_encoding::NumericEncoding};

fn is_stack_depth_reasonable(current_stack_depth: u32, opcode: &Opcode) -> bool {
    current_stack_depth >= (opcode.get_minimum_stack_depth() as u32)
}

fn is_program_counter_reasonable_after_executing(current_program_counter: u32, read_stack_value_1: u32, read_stack_value_2: u32, opcode: &Opcode, program_memory_length: u32) -> bool {
    match opcode {
        Opcode::Stop | Opcode::Return | Opcode::Error => {
            // Stop does not move pc => NoError
            // Return and Error move pc to location of Stop => always NoError
            
            true
        },
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
            // pc is moved next
            current_program_counter + 1 < program_memory_length
        },
        Opcode::Div | Opcode::Mod => {
            let b = read_stack_value_2;
            b != 0 && current_program_counter + 1 < program_memory_length // divisor must be non-zero and next pc is set to be pc + 1
        },
        Opcode::Jump => {
            let destination = read_stack_value_1;
            destination < program_memory_length // it jumps to destination, so destination must be valid
        },
        Opcode::Jumpi => {
            let (destination, condition) = (read_stack_value_1, read_stack_value_2);
            (
                ((condition == 0) as u32) * (current_program_counter + 1) // if condition is zeo then new_pc = pc + 1
                + (1 - (condition == 0) as u32) * destination // else new_pc = destination
            ) < program_memory_length // check new_pc < program_memory_length
        },
    }
}

fn compute_next_program_counter(
    current_program_counter: u32,
    read_stack_value_1: u32,
    read_stack_value_2: u32,
    opcode: &Opcode,
    error_index: u32,
    stop_index: u32,
    is_error: bool,
    is_not_error: bool,
) -> u32 {
    // now output result
    match opcode {
        Opcode::Stop => {
            (is_error as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * current_program_counter // else, program counter is unchanged
        },
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
            (is_error as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * (current_program_counter + 1) // else, program counter is set to be pc + 1
        },
        Opcode::Return | Opcode::Error => {
            (is_error as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * stop_index // else, program counter is set to be stop_index
        },
        Opcode::Jump => {
            let destination = read_stack_value_1;
            (is_error as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * destination // else, program counter is set to be destination
        },
        Opcode::Jumpi => {
            let (destination, condition) = (read_stack_value_1, read_stack_value_2);
            (is_error as u32) * error_index // in case of error, pc jumps to error_index
            + (is_not_error as u32) * (!(condition == 0) as u32) * destination // if condition != 0, jump to destination
            + (is_not_error as u32) * ((condition == 0) as u32) * (current_program_counter + 1) // if condition == 0, jump to pc + 1
        },
    }
}

fn compute_next_stack_depth(
    current_stack_depth: u32,
    opcode: &Opcode,
    is_error: bool,
    is_not_error: bool,
) -> u32 {
    match opcode {
        Opcode::Stop | Opcode::Swap1 => {
            (is_error as u32) * (current_stack_depth + 1) // push error code
            + (is_not_error as u32) * current_stack_depth // stack unchanged
        },
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod | Opcode::Pop | Opcode::Jump | Opcode::Return | Opcode::Error => {
            (is_error as u32) * (current_stack_depth + 1) // if error then push error code
            + (is_not_error as u32) * (current_stack_depth - 1) // (pop 2 elements and push 1) or (just pop 1 element as Opcode::Pop or Opcode::Jump)
        },
        Opcode::Push4 | Opcode::Dup2 => {
            current_stack_depth + 1 // push 1 more element either error or not
        },
        Opcode::Jumpi => {
            (is_error as u32) * (current_stack_depth + 1) // if error then push error code
            + (is_not_error as u32) * (current_stack_depth - 2) // (pop 2 elements for destination and condition)
        }
    }
}


pub fn compute_next_state(
    current_stack_depth: u32, // hidden
    current_program_counter: u32, // hidden
    read_stack_value_1: u32, // hidden // top value of the stack
    read_stack_value_2: u32, // hidden // 2nd top value of the stack
    opcode: u32, // public
    program_memory_length: u32, // public constant
    error_index: u32, // public constant
    stop_index: u32, // public constant
) -> (u32, u32) /* next state includes (stack_depth, program_counter) */ {
    // computing is_stack_depth_reasonable
    // notice that opcode is public => opcode.get_stack_depth_minimum() is publicly known
    let opcode = Opcode::from_u32(opcode);
    let is_stack_depth_reasonable = is_stack_depth_reasonable(current_stack_depth, &opcode);

    // computing is_program_counter_reasonable_after_executing
    // program_memory_length is considered a fixed constant when conducting proof
    // computation is depending on implementation of trait OpcodeExecutionChecker for opcode
    let is_program_counter_reasonable_after_executing = is_program_counter_reasonable_after_executing(
        current_program_counter, 
        read_stack_value_1,
        read_stack_value_2,
        &opcode,
        program_memory_length,
    );

    let is_not_error = is_stack_depth_reasonable && is_program_counter_reasonable_after_executing;
    let is_error = !is_not_error;

    (
        compute_next_stack_depth(
            current_stack_depth,
            &opcode,
            is_error,
            is_not_error
        ),
        compute_next_program_counter(
            current_program_counter, 
            read_stack_value_1,
            read_stack_value_2,
            &opcode,
            error_index,
            stop_index,
            is_error,
            is_not_error,
        )
    )
}