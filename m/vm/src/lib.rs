pub const VERSION: &str = "0.0.1";
pub mod icode;
pub mod syscall;
pub mod value;
pub use {
    crate::vm::Vm,
    icode::{ICode, Instr, StringInfo},
    value::{Value, ValueTypeInfo},
};

mod debugger;
mod vm;

use {
    collection::{Deq, Entry, Map},
    job::Job,
    std::{borrow::BorrowMut, convert::TryFrom, io, mem},
};

error::Error! {
    Msg = &'static str
    Message = String
    Var = std::env::VarError
    Io = io::Error
    Utf8Error = std::string::FromUtf8Error
    Job = job::Error
    Debugger = debugger::Error
    Fmt = std::fmt::Error
    BufferSd = buf::sd2::Error
}
use error::{ltrace, soft_todo};
use error::{te, temg, terr};
