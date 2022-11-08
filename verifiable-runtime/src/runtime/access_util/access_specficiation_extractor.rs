use crate::runtime::access_util::access_specification::AccessSpecification;
use crate::runtime::constants::{MAXIMUM_NUM_ACCESSES_PER_OPCODE};

pub trait AccessSpecificationExtractor {
    fn get_access_specification(&self) -> [AccessSpecification; MAXIMUM_NUM_ACCESSES_PER_OPCODE];
}