pub const VERSION: &str = "0.0.1";

mod load_icode;
pub mod sd;

pub use {
    error::te,
    load_icode::{list_func, load_compiler, load_icode, make_vm, make_vm_call},
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
    Log = log::SetLoggerError
}

pub fn init() -> Result<()> {
    te!(pretty_env_logger::try_init());
    Ok(())
}
