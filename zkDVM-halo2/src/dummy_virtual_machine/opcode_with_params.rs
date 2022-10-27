use super::{opcode::Opcode, constants::MAXIMUM_NUM_OPCODE_PARAMS_PER_OPCODE};

// this struct is used to put inside program memory
// it includes the opcode and possibly an additional param (like push value)
#[derive(Clone)]
pub struct OpcodeWithParams {
    opcode: Opcode,
    params: [Option<u32>; MAXIMUM_NUM_OPCODE_PARAMS_PER_OPCODE], // Some(param as u32) if there is some param. Otherwise, None
}

impl OpcodeWithParams {
    pub fn new(opcode: Opcode, params: &[Option<u32>; MAXIMUM_NUM_OPCODE_PARAMS_PER_OPCODE]) -> Self {
        Self {
            opcode: opcode,
            params: *params,
        }
    }

    pub fn get_opcode(&self) -> Opcode {
        self.opcode.clone()
    }

    pub fn get_param(&self, index: usize) -> Option<u32> {
        self.params[index]
    }

    pub fn get_all_params(&self) -> [Option<u32>; MAXIMUM_NUM_OPCODE_PARAMS_PER_OPCODE] {
        self.params
    }
}