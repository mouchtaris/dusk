use std::{
    fmt,
    ops::{Add, Sub},
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

    pub fn wrap<U>(self, p: (U, U)) -> T
    where
        U: Add<T, Output = T> + Sub<T, Output = T>,
    {
        match self {
            Plus(t) => p.0 + t,
            Minus(t) => p.1 - t,
        }
    }

    pub fn rebase<U, R>(self, p: (Signed<U>, Signed<U>)) -> Signed<R>
    where
        U: Add<T, Output = R> + Sub<T, Output = R>,
    {
        match self {
            Plus(a) => match p.0 {
                Plus(b) => Plus(b + a),
                Minus(b) => Minus(b - a),
            },
            Minus(a) => match p.1 {
                Plus(b) => Plus(b - a),
                Minus(b) => Minus(b + a),
            },
        }
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
