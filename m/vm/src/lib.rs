pub const VERSION: &str = "0.0.1";
pub mod syscall;
pub use {
    crate::vm::Vm,
    icode::{ICode, Instr, StringInfo},
    value::{Value, ValueTypeInfo},
};

mod icode;
mod value;
mod vm;

use {
    collection::{Deq, Entry, Map},
    std::{borrow::BorrowMut, convert::TryFrom, io},
};

error::Error! {
    Msg = &'static str
    Message = String
    Var = std::env::VarError
    Io = io::Error
    Utf8Error = std::string::FromUtf8Error
}
use error::{ldebug, ltrace, soft_todo};
use error::{te, temg, terr};
