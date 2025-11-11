pub const VERSION: &str = "0.0.1";
pub mod debugger;
pub mod icode;
pub mod syscall;
pub mod to_sh;
pub mod value;
pub use {
    icode::{ICode, Instr, Instrs, StringInfo, Strings},
    value::{Value, ValueTypeInfo},
    vm::{Vm, DEBUG_STACK_SIZE},
};

mod vm;

use {
    collection::{Deq, Entry, Map},
    job::Job,
    std::{borrow::BorrowMut, convert::TryFrom, io},
};

error::Error! {
    Msg = &'static str
    Message = String
    Var = std::env::VarError
    Io = io::Error
    FromUtf8 = std::string::FromUtf8Error
    Utf8 = std::str::Utf8Error
    Job = job::Error
    Debugger = debugger::Error
    Fmt = std::fmt::Error
    BufferSd = buf::sd2::Error
    ParseInt = std::num::ParseIntError
}
use error::{ltrace, soft_todo};
use error::{te, temg, terr};
