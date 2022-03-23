pub const VERSION: &str = "0.0.1";

use {
    ::error::ltrace,
    ::lexpop::{any, as_fn, either, exact, fn_, lexpop, one_and_any},
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
    ($($name:ident $( ( $($var:ident),* ) )? ),*) => {
        $(
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $name<'i>(pub T<'i> $(, $(pub $var<'i>),* )? );
        )*
    };
}
macro_rules! tokens {
    ($($name:ident),*) => {
        name![$($name),*];
        either![Tok($($name),*)];
    }
}

type T<'i> = &'i str;
tokens![
    Whsp,
    Nada,
    Kwd,
    Idnt,
    IdntNe,
    AbsPath,
    RelPath,
    LongOpt,
    ShortOpt,
    LineComment,
    RawString,
    Natural
];
use lexpop::lex::fat as rawstring;
lexpop![natural, any(|| fn_(digit))];
lexpop![longopt, one_and_any(exact("--"), ident)];
lexpop![shortopt, one_and_any(exact("-"), ident)];
lexpop![abspath, one_and_any(exact("/"), ident)];
lexpop![relpath, one_and_any(exact("./"), ident)];
lexpop![ident, one_and_any(fn_(ident_init), || fn_(ident_rest))];
lexpop![
    ident_no_eq,
    one_and_any(fn_(ident_init), || fn_(ident_rest_no_eq))
];
lexpop![whsp, fn_(char::is_whitespace)];
lexpop![
    linecomment,
    one_and_any(fn_(eq('#')), || fn_(not(eq('\n'))))
];
lexpop![
    kwd,
    either(
        '{',
        either(
            '}',
            either(
                '=',
                either(
                    '$',
                    either(
                        '"',
                        either(
                            ';',
                            either(
                                '@',
                                either(
                                    '<',
                                    either(
                                        '>',
                                        either(
                                            '!',
                                            either(exact("let"), either(exact("def"), exact("if")))
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    )
];

pub const TOK_NADA: Tok<'static> = Tok::Nada(Nada(""));

pub type Offset = usize;
pub type Spanned<T> = (Offset, T, Offset);

#[derive(Clone)]
pub struct LexState<'i> {
    inp: T<'i>,
    pos: usize,
}

pub struct Lex<'i> {
    pub state: LexState<'i>,
}
impl<'i> Lex<'i> {
    pub fn new(inp: &'i str) -> Self {
        let state = LexState::new(inp);
        Self { state }
    }
}
impl<'i> Iterator for Lex<'i> {
    type Item = IterItem<LexState<'i>>;
    fn next(&mut self) -> Item<Self> {
        self.state.next()
    }
}

fn to_str<'i>(tok: Option<&'i Spanned<Tok>>) -> &'i str {
    tok.as_ref().unwrap().1.as_ref()
}

fn ident_or_kwd<'i>(s: &mut LexState<'i>) -> Item<LexState<'i>> {
    let mut s0 = s.clone();

    match s.mtch(ident(), Idnt) {
        None => None,

        tid @ Some(_) => {
            let id = to_str(tid.as_ref());

            match s0.mtch(kwd(), Kwd) {
                None => tid,

                tkwd @ Some(_) => {
                    let kwd = to_str(tkwd.as_ref());

                    if kwd.len() == id.len() {
                        tkwd
                    } else {
                        tid
                    }
                }
            }
        }
    }
}

impl<'i> Lex<'i> {}

impl<'i> Iterator for LexState<'i> {
    type Item = Spanned<Tok<'i>>;
    fn next(&mut self) -> Item<Self> {
        let Self { inp, pos } = self;
        let p = *pos;

        if p == inp.len() {
            return None;
        }

        // Eat up comments and whitespace
        self.mtch(whsp(), Whsp);
        loop {
            let lc = self.mtch(any(|| as_fn(linecomment())), LineComment);
            self.mtch(whsp(), Whsp);
            if lc.is_none() {
                break;
            }
        }

        let iok = None
            .or_else(|| self.mtch(natural(), Natural))
            .or_else(|| self.mtch(abspath(), AbsPath))
            .or_else(|| self.mtch(relpath(), RelPath))
            .or_else(|| self.mtch(longopt(), LongOpt))
            .or_else(|| self.mtch(shortopt(), ShortOpt))
            .or_else(|| self.mtch(rawstring(), RawString))
            .or_else(|| ident_or_kwd(self))
            .or_else(|| self.mtch(kwd(), Kwd))
            .or_else(|| None);

        let r = iok;
        ltrace!("rt: -> {:?}", r);
        r
    }
}

type IterItem<S> = <S as Iterator>::Item;
pub type Item<S> = Option<IterItem<S>>;

impl<'i> LexState<'i> {
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

fn digit(c: char) -> bool {
    c.is_ascii_digit()
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

fn eq(c: char) -> impl Fn(char) -> bool {
    move |x| c == x
}
fn not<A, P>(mut pred: P) -> impl FnMut(A) -> bool
where
    P: FnMut(A) -> bool,
{
    move |x| !pred(x)
}

impl<'i> AsRef<str> for Tok<'i> {
    fn as_ref(&self) -> &str {
        use Tok as t;
        match self {
            t::Nada(Nada(s))
            | t::Natural(Natural(s))
            | t::RawString(RawString(s))
            | t::LineComment(LineComment(s))
            | t::LongOpt(LongOpt(s))
            | t::ShortOpt(ShortOpt(s))
            | t::AbsPath(AbsPath(s))
            | t::RelPath(RelPath(s))
            | t::IdntNe(IdntNe(s))
            | t::Idnt(Idnt(s))
            | t::Whsp(Whsp(s))
            | t::Kwd(Kwd(s)) => s,
        }
    }
}
