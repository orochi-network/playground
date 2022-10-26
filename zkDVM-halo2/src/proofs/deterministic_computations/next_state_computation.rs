use crate::{dummy_virtual_machine::{
    opcode::{Opcode},
    stack_requirement::StackRequirement, constants::MAXIMUM_NUM_READS_PER_OPCODE,
}, utils::numeric_encoding::NumericEncoding, proofs::proof_types::{
    p_stack_value::PStackValue, 
    p_stack_depth::PStackDepth, 
    p_numeric_encoding::PNumericEncoding, 
    p_program_counter::PProgramCounter,
    p_program_memory_location::PProgramMemoryLocation, 
    p_opcode::POpcode
}};

fn is_stack_depth_reasonable(
    current_stack_depth: &PStackDepth, 
    opcode: &Opcode
) -> bool {
    current_stack_depth.to_u32() >= (opcode.get_minimum_stack_depth() as u32)
}

fn is_program_counter_reasonable_after_executing(
    current_program_counter: &PProgramCounter, 
    read_stack_values: &[PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], 
    opcode: &Opcode, 
    program_memory_length: u32,
) -> bool {
    match opcode {
        Opcode::Stop | Opcode::Return | Opcode::Error => {
            // Stop does not move pc => NoError
            // Return and Error move pc to location of Stop => always NoError
            
            true
        },
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
            // pc is moved next
            current_program_counter.to_u32() + 1 < program_memory_length
        },
        Opcode::Div | Opcode::Mod => {
            let b = read_stack_values[1];
            b.to_u32() != 0 && current_program_counter.to_u32() + 1 < program_memory_length // divisor must be non-zero and next pc is set to be pc + 1
        },
        Opcode::Jump => {
            let destination = &read_stack_values[0];
            destination.to_u32() < program_memory_length // it jumps to destination, so destination must be valid
        },
        Opcode::Jumpi => {
            let (destination, condition) = &(read_stack_values[0], read_stack_values[1]);
            (
                ((condition.to_u32() == 0) as u32) * (current_program_counter.to_u32() + 1) // if condition is zeo then new_pc = pc + 1
                + (1 - (condition.to_u32() == 0) as u32) * destination.to_u32() // else new_pc = destination
            ) < program_memory_length // check new_pc < program_memory_length
        },
    }
}

fn compute_next_program_counter(
    current_program_counter: &PProgramCounter,
    read_stack_values: &[PStackValue; MAXIMUM_NUM_READS_PER_OPCODE],
    opcode: &Opcode,
    error_index: &PProgramMemoryLocation,
    stop_index: &PProgramMemoryLocation,
    is_error: bool,
    is_not_error: bool,
) -> PProgramCounter {
    // now output result
    PProgramCounter::from_u32(
        match opcode {
            Opcode::Stop => {
                (is_error as u32) * error_index.to_u32() // in case of error, pc jumps to error_index
                + (is_not_error as u32) * current_program_counter.to_u32() // else, program counter is unchanged
            },
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod | Opcode::Push4 | Opcode::Dup2 | Opcode::Pop | Opcode::Swap1 => {
                (is_error as u32) * error_index.to_u32() // in case of error, pc jumps to error_index
                + (is_not_error as u32) * (current_program_counter.to_u32() + 1) // else, program counter is set to be pc + 1
            },
            Opcode::Return | Opcode::Error => {
                (is_error as u32) * error_index.to_u32() // in case of error, pc jumps to error_index
                + (is_not_error as u32) * stop_index.to_u32() // else, program counter is set to be stop_index
            },
            Opcode::Jump => {
                let destination = &read_stack_values[0];
                (is_error as u32) * error_index.to_u32() // in case of error, pc jumps to error_index
                + (is_not_error as u32) * destination.to_u32() // else, program counter is set to be destination
            },
            Opcode::Jumpi => {
                let (destination, condition) = &(read_stack_values[0], read_stack_values[1]);
                (is_error as u32) * error_index.to_u32() // in case of error, pc jumps to error_index
                + (is_not_error as u32) * (!(condition.to_u32() == 0) as u32) * destination.to_u32() // if condition != 0, jump to destination
                + (is_not_error as u32) * ((condition.to_u32() == 0) as u32) * (current_program_counter.to_u32() + 1) // if condition == 0, jump to pc + 1
            },
        }
    )
}

fn compute_next_stack_depth(
    current_stack_depth: &PStackDepth,
    opcode: &Opcode,
    is_error: bool,
    is_not_error: bool,
) -> PStackDepth {
    PStackDepth::from_u32(
        match opcode {
            Opcode::Stop | Opcode::Swap1 => {
                (is_error as u32) * (current_stack_depth.to_u32() + 1) // push error code
                + (is_not_error as u32) * current_stack_depth.to_u32() // stack unchanged
            },
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod | Opcode::Pop | Opcode::Jump | Opcode::Return | Opcode::Error => {
                (is_error as u32) * (current_stack_depth.to_u32() + 1) // if error then push error code
                + (is_not_error as u32) * (current_stack_depth.to_u32() - 1) // (pop 2 elements and push 1) or (just pop 1 element as Opcode::Pop or Opcode::Jump)
            },
            Opcode::Push4 | Opcode::Dup2 => {
                current_stack_depth.to_u32() + 1 // push 1 more element either error or not
            },
            Opcode::Jumpi => {
                (is_error as u32) * (current_stack_depth.to_u32() + 1) // if error then push error code
                + (is_not_error as u32) * (current_stack_depth.to_u32() - 2) // (pop 2 elements for destination and condition)
            }
        }
    )
}

// fn compute_next_stack_written_values(
//     opcode: &Opcode,
// ) -> [PStackValue; MAXIMUM_NUM_WRITES_PER_OPCODE] {
//     // input:  [read_stack_value_1,  read_stack_value_2,  read_stack_value_3, ...
//     // output: [write_stack_value_1, write_stack_value_2, ...
//     todo!();
// }


pub fn compute_next_state(
    current_stack_depth: &PStackDepth, // hidden
    current_program_counter: &PProgramCounter, // hidden
    read_stack_values: &[PStackValue; MAXIMUM_NUM_READS_PER_OPCODE], // read_stack_values
    opcode: &POpcode, // public
    program_memory_length: u32, // public constant
    error_index: &PProgramMemoryLocation, // public constant
    stop_index: &PProgramMemoryLocation, // public constant
) -> (PStackDepth, PProgramCounter) /* next state includes (stack_depth, program_counter, (write_stack_value_1, write_stack_value_2)) */ {
    // computing is_stack_depth_reasonable
    // notice that opcode is public => opcode.get_stack_depth_minimum() is publicly known
    let opcode = Opcode::from_u32(opcode.to_u32());
    let is_stack_depth_reasonable = is_stack_depth_reasonable(current_stack_depth, &opcode);

    // computing is_program_counter_reasonable_after_executing
    // program_memory_length is considered a fixed constant when conducting proof
    // computation is depending on implementation of trait OpcodeExecutionChecker for opcode
    let is_program_counter_reasonable_after_executing = is_program_counter_reasonable_after_executing(
        &current_program_counter, 
        &read_stack_values,
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
            read_stack_values,
            &opcode,
            error_index,
            stop_index,
            is_error,
            is_not_error,
        ),
        // compute_next_stack_written_values(&opcode)
    )
}