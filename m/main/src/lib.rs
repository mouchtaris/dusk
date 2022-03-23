pub const VERSION: &str = "0.0.1";

pub use std::{collections, fmt, fs, io, iter, prelude, slice, str, string, u32, u8, usize, vec};

pub mod sd;
error::Error! {
    Msg = String
    Io = io::Error
    Parse = parse::Error
    Vm = vm::Error
    Compile = compile::Error
    CBor = sd::CborError
}
