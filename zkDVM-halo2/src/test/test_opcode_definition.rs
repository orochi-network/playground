use crate::dummy_virtual_machine::{
    opcode::{Opcode}, 
    numeric_encoding::NumericEncoding, error_code::ErrorCode
};

pub struct TestOpcodeDefinition {
    
}

impl TestOpcodeDefinition {
    pub fn test() {
        assert_eq!(Opcode::from_u32(0xfe), Opcode::Error);
        println!("{:?}", Opcode::from_u32(0x81));
        println!("{:?}", ErrorCode::from_u32(0x03));
    }
}