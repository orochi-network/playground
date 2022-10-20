use crate::opcode_definition::{Opcode, ErrorCode, NumericEncoding};

pub struct TestOpcodeDefinition {
    
}

impl TestOpcodeDefinition {
    pub fn test() {
        assert_eq!(Opcode::from_u32(0xfe), Opcode::Error);
        println!("{:?}", Opcode::from_u32(0x81));
        println!("{:?}", ErrorCode::from_u32(0x03));
    }
}