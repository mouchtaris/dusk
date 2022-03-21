pub const VERSION: &str = "0.0.1";

pub use std::collections::{
    hash_map::{Entry, HashMap as Map},
    VecDeque as Deq,
};

pub trait OptionInspect<T> {
    fn inspct<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);
}

impl<T> OptionInspect<T> for Option<T> {
    fn inspct<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        match self {
            Some(v) => {
                f(&v);
                Some(v)
            }
            None => None,
        }
    }
}
