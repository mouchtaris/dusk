pub const VERSION: &str = "0.0.1";
mod icode;
mod value;
mod vm;
pub use {
    crate::vm::Vm,
    icode::{ICode, Instr, StringInfo},
    value::{Value, ValueTypeInfo},
};
use {
    collection::{Deq, Entry, Map},
    std::{
        borrow::{Borrow, BorrowMut},
        convert::TryFrom,
    },
};

error::Error! {
    Msg = &'static str
    Message = String
    Var = std::env::VarError
}
use error::te;
use error::{ldebug, ltrace, soft_todo};
