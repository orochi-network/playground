pub mod helper;
mod runtime;
pub mod stark;
pub mod utils;

pub use runtime::dvm;
pub const DIGEST_SIZE: usize = 2;
