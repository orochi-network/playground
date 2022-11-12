use crate::runtime::constants::MAXIMUM_NUM_OPCODE_PARAMS_PER_OPCODE;
use crate::runtime::opcode_util::opcode::Opcode;

// this struct is used to put inside program memory
// it includes the opcode and possibly an additional param (like push value)
#[derive(Clone, Debug)]
pub struct OpcodeWithImmediateValue {
    opcode: Opcode,
    immediate_value: Option<u32>, // Some(param as u32) if there is some immediate value. Otherwise, None
}

impl OpcodeWithImmediateValue {
    pub fn new(opcode: Opcode, immediate_value: Option<u32>) -> Self {
        Self {
            opcode: opcode,
            immediate_value: immediate_value.clone(),
        }
    }

    pub fn get_opcode(&self) -> Opcode {
        self.opcode.clone()
    }

    pub fn get_immediate_value(&self) -> Option<u32> {
        self.immediate_value
    }
}