use crate::runtime::error_code_util::error_code::ErrorCode;
use crate::runtime::opcode_util::opcode::Opcode;
use crate::utils::numeric_encoding::NumericEncoding;

pub struct TestOpcodeDefinition {
    
}

impl TestOpcodeDefinition {
    pub fn test() {
        assert_eq!(Opcode::from_u32(0xfe), Opcode::Error);
        println!("{:?}", Opcode::from_u32(0x81));
        println!("{:?}", ErrorCode::from_u32(0x03));
    }
}