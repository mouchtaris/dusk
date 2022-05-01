use std::{
    fmt,
    ops::{Add, Range, Sub},
};

#[derive(Copy, Clone, Debug)]
pub enum Signed<T> {
    Plus(T),
    Minus(T),
}
use Signed::Minus;
use Signed::Plus;

impl<T> Signed<T> {
    pub fn into(self) -> T {
        match self {
            Plus(t) => t,
            Minus(t) => t,
        }
    }

    pub fn resolve<U>(&self, range: Range<U>) -> Self
    where
        U: Add<T, Output = T> + Sub<T, Output = T>,
        T: Copy,
    {
        Plus(match self {
            &Plus(t) => range.start + t,
            &Minus(t) => range.end - t,
        })
    }
}

impl<T: fmt::Display> fmt::Display for Signed<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (sign, num) = match self {
            Plus(n) => ('+', n),
            Minus(n) => ('-', n),
        };
        write!(f, "{}{}", sign, num)
    }
}

impl<T: Default> Default for Signed<T> {
    fn default() -> Self {
        Plus(<_>::default())
    }
}
