pub const VERSION: &str = "0.0.1";

mod load_icode;
pub mod sd;

pub use {
    error::te,
    load_icode::{
        args_get_input, args_get_output, list_func, load_compiler, load_icode, make_vm,
        make_vm_call, read_compiler,
    },
    std::{
        boxed, collections, fmt, fs, io, iter, prelude, slice, str, string, u32, u8, usize, vec,
    },
};

error::Error! {
    Msg = String
    Io = io::Error
    Parse = parse::Error
    Vm = vm::Error
    VmDebugger = vm::debugger::Error
    Compile = compile::Error
    CBor = sd::CborError
    Utf8 = std::str::Utf8Error
    Log = log::SetLoggerError
}

pub fn init() -> Result<()> {
    pretty_env_logger::init();
    Ok(())
}
