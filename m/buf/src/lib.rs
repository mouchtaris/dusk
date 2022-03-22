pub const VERSION: &str = "0.0.1";

mod mem_take;
mod string_buf;

pub use {mem_take::MemTake, string_buf::StringBuf};
