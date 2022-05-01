use {
    super::{value, Job, Result, Value, Vm},
    error::{ldebug, te},
};

pub const SPAWN: usize = usize::MAX;
pub const ARG_SLICE: usize = usize::MAX - 1;

mod argslice;
mod spawn;
pub use {argslice::argslice, spawn::spawn};
