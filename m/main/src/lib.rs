pub const VERSION: &str = "0.0.1";

mod load_icode;
pub mod sd;

pub use {
    load_icode::load_icode,
    std::{
        boxed, collections, fmt, fs, io, iter, prelude, slice, str, string, u32, u8, usize, vec,
    },
};

error::Error! {
    Msg = String
    Io = io::Error
    Parse = parse::Error
    Vm = vm::Error
    Compile = compile::Error
    CBor = sd::CborError
    Utf8 = std::str::Utf8Error
}
