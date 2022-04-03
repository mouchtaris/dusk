pub const VERSION: &str = "0.0.1";

pub mod sd2;
mod string_buf;
mod string_buf_iter;

pub use {string_buf::StringBuf, string_buf_iter::StringBufIterator};
