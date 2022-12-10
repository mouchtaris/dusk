pub const VERSION: &str = "0.0.1";

pub mod errors;
pub mod sd;

mod exec_common;
mod load_icode;

pub use {
    error::te,
    exec_common::run_main,
    load_icode::{
        args_get_input, args_get_output, list_func, load_compiler, load_icode, make_vm,
        make_vm_call, read_compiler,
    },
    std::{
        boxed, collections, env, fmt, fs, io, iter, prelude, slice, str, string, u32, u8, usize,
        vec,
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
    Var = env::VarError
}

pub fn init() -> Result<()> {
    pretty_env_logger::init();
    Ok(())
}
