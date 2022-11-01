use crate::runtime::access_util::access_specification::AccessSpecification;
use crate::runtime::constants::{MAXIMUM_NUM_READS_PER_OPCODE, MAXIMUM_NUM_WRITES_PER_OPCODE};

pub trait AccessSpecificationExtractor {
    fn get_access_specification(&self) -> (
        [AccessSpecification; MAXIMUM_NUM_READS_PER_OPCODE],
        [AccessSpecification; MAXIMUM_NUM_WRITES_PER_OPCODE]
    );
}