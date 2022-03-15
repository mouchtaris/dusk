pub const VERSION: &str = "0.0.1";

macro_rules! name0 {
    ($($name:ident = $($v:ident)*),*) => {
        $(
        pub type $name<'i> = ($($v<'i>),*);
        )*
    };
}
macro_rules! name {
    ($($name:ident = $($v:ident)*),*) => {
        $(
        #[derive(Debug)]
        pub struct $name<'i>(pub ($($v<'i>),*));
        impl <'i> From<($($v<'i>),*)> for $name<'i> {
            fn from(v: ($($v<'i>),*)) -> Self {
                Self(v)
            }
        }
        )*
    };
}
macro_rules! either {
    (
        $name:ident
        $($v:ident)*
    ) => {
        #[derive(Debug)]
        pub enum $name<'i> {
            $( $v($v<'i>), )*
        }
        $(
        impl <'i> From<$v<'i>> for $name<'i> {
            fn from(v: $v<'i>) -> Self {
                Self::$v(v)
            }
        }
        )*
    };
}

pub mod ast;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub dust);

use dust::Token;

pub type ParseError<'s> = lalrpop_util::ParseError<usize, Token<'s>, &'s str>;

pub type Result<'s, T> = std::result::Result<T, ParseError<'s>>;

pub fn parse(s: &str) -> Result<ast::Script> {
    dust::ScriptParser::new().parse(s)
}
