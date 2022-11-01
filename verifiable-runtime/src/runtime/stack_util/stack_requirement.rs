pub trait StackRequirement {
    // return the minimum depth of the stack to ensure whether the corresponding opcode executed correctly
    fn get_num_stack_params(&self) -> usize;

    // return the number of params needed from the stack for satisfying the opcode's execution
    fn get_minimum_stack_depth(&self) -> usize;
}