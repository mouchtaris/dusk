pub const VERSION: &str = "0.0.1";

pub mod cli;
pub mod errors;
pub mod sd;

mod exec_common;
mod load_icode;

pub use {
    compile::{self, Compiler},
    error::{te, IntoResult},
    exec_common::{run_app, run_main},
    load_icode::{
        args_get_input, args_get_output, compile_file, compile_from_input, compile_input_with_base,
        list_func, load_compiler, load_icode, make_vm, make_vm_call, read_compiler,
        script_call_getret,
    },
    std::{
        boxed, collections, env, fmt, fs, io, iter, prelude, slice, str, string, u32, u8, usize,
        vec,
    },
    vm,
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
    Job = job::Error
}

pub fn init() -> Result<()> {
    pretty_env_logger::init();
    Ok(())
}
