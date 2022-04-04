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

pub trait Recollect: Sized
where
    Self: IntoIterator,
{
    fn recollect<U>(self) -> U
    where
        U: std::iter::FromIterator<Self::Item>,
    {
        self.into_iter().collect()
    }

    fn to_vec(self) -> Vec<Self::Item> {
        self.recollect()
    }

    fn sorted(self) -> Vec<Self::Item>
    where
        Self::Item: Ord,
    {
        let mut v = self.to_vec();
        v.sort();
        v
    }
}
impl<S: IntoIterator> Recollect for S {}
