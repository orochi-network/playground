use crate::runtime::error_code_util::error_code::ErrorCode;
// use crate::runtime::trace::raw_execution_trace::RawExecutionTrace;

pub trait Execution {
    fn execute(&mut self, execution_length: usize) -> (u32, ErrorCode);
}