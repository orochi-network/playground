use crate::{opcode::Opcode, utils::numeric_encoding::NumericEncoding, runtime::error_code::ErrorCode};


pub struct TestOpcodeDefinition {
    
}

impl TestOpcodeDefinition {
    pub fn test() {
        assert_eq!(Opcode::from_u32(0xfe), Opcode::Error);
        println!("{:?}", Opcode::from_u32(0x81));
        println!("{:?}", ErrorCode::from_u32(0x03));
    }
}