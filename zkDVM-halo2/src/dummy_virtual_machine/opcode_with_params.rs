use super::opcode::Opcode;

// this struct is used to put inside program memory
// it includes the opcode and possibly an additional param (like push value)
#[derive(Clone)]
pub struct OpcodeWithParams {
    opcode: Opcode,
    param: Option<u32>, // Some(param as u32) if there is some param. Otherwise, None
}

impl OpcodeWithParams {
    pub fn new(opcode: Opcode, param: Option<u32>) -> Self {
        Self {
            opcode: opcode,
            param: param,
        }
    }

    pub fn get_opcode(&self) -> Opcode {
        self.opcode.clone()
    }

    pub fn get_param(&self) -> Option<u32> {
        self.param
    }
}