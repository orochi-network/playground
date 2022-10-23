use super::{error_code::ErrorCode, raw_execution_trace::RawExecutionTrace};

pub trait Execution {
    fn execute(&mut self, execution_length: usize) -> (u32, ErrorCode, RawExecutionTrace);
}