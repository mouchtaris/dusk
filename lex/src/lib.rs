pub const VERSION: &str = "0.0.1";

use {
    ::error::ltrace,
    ::lexpop::{chain, exact, from_fn, lexpop},
};

macro_rules! either {
    ($($name:ident ( $($var:ident),* ) ),*) => {
        $(
        ::either::either![
            #[derive(Debug, Clone, Eq, PartialEq)]
            pub Tok<'i>,
            $($var<'i>),*
        ];
        )*
    };
}
macro_rules! name {
    ($($name:ident ( $($var:ident),* ) ),*) => {
        $(
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $name<'i>($(pub $var<'i>),*);
        )*
    };
}

type T<'i> = &'i str;
name![Kwd(T), Op(T), Nada(T), Whsp(T), Idnt(T), IdntNe(T)];

either![Tok(Whsp, Nada, Kwd, Op, Idnt, IdntNe)];

lexpop![whsp, from_fn(char::is_whitespace)];
lexpop![kwd, exact("let")];
lexpop![ident, chain(from_fn(ident_init), from_fn(ident_rest))];
lexpop![
    ident_no_eq,
    chain(from_fn(ident_init), from_fn(ident_rest_no_eq))
];

pub const TOK_NADA: Tok<'static> = Tok::Nada(Nada(""));

pub type Offset = usize;
pub type Spanned<T> = (Offset, T, Offset);

pub struct Lex<'i> {
    inp: T<'i>,
    pos: usize,
}

impl<'i> Iterator for Lex<'i> {
    type Item = Spanned<Tok<'i>>;
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inp, pos } = self;
        let p = *pos;

        if p == inp.len() {
            return None;
        }

        let ws = self.mtch(whsp(), Whsp);
        ltrace!("ws: {:?}", ws);

        let r = self.mtch(kwd(), Kwd).or_else(|| self.mtch(ident(), Idnt));
        ltrace!("r: {:?}", r);
        r
    }
}

impl<'i> Lex<'i> {
    pub fn new(inp: &'i str) -> Self {
        Self { inp, pos: 0 }
    }

    fn mtch<M, C, T>(&mut self, mut matcher: M, ctor: C) -> Option<Spanned<Tok<'i>>>
    where
        M: lexpop::Prop,
        C: FnOnce(&'i str) -> T,
        T: Into<Tok<'i>>,
    {
        let Self { pos, inp } = self;
        let p = *pos;
        let range = &inp[p..];

        let n = matcher.match_range::<3, _>(range.chars());
        match n {
            0 => None,
            n => {
                let t = ctor(&range[0..n]).into();
                let sp = (p, t, p + n);
                *pos += n;
                Some(sp)
            }
        }
    }
}

fn ident_init(c: char) -> bool {
    c.is_alphabetic()
}

fn ident_rest(c: char) -> bool {
    ident_init(c) || c.is_digit(10) || ":.,_=/-".find(c).is_some()
}

fn ident_rest_no_eq(c: char) -> bool {
    ident_init(c) || c.is_digit(10) || ":.,_/-".find(c).is_some()
}

impl<'i> AsRef<str> for Tok<'i> {
    fn as_ref(&self) -> &str {
        use Tok as t;
        match self {
            t::IdntNe(IdntNe(s))
            | t::Idnt(Idnt(s))
            | t::Whsp(Whsp(s))
            | t::Op(Op(s))
            | t::Nada(Nada(s))
            | t::Kwd(Kwd(s)) => s,
        }
    }
}
