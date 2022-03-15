use std::collections::VecDeque as Deq;
use std::marker::PhantomData as __;

pub type Script<'i> = Vec<Item<'i>>;
pub type Text<'i> = &'i str;
pub type OptText<'i> = Option<Text<'i>>;
pub type Path<'i> = (bool, Deq<Term<'i>>);
pub type Cwd<'i> = Option<Path<'i>>;
pub type Params<'i> = Vec<Ident<'i>>;

name0! {
    Block = Script,
    File = Path
}
name! {
    Dot = Text,
    DotDot = Text,
    Variable = Ident,
    Local = Ident,
    Ident = Text,
    LongOpt = Text OptText,
    ShortOpt = Text OptText,
    VarShortOpt = Text,
    Pipe = Dispatch
    , Let = Ident Body
    , Def = Ident Params Body
}

#[derive(Debug)]
pub struct Command<'i> {
    pub cwd: Cwd<'i>,
    pub dispatch: Dispatch<'i>,
    pub args: Vec<Value<'i>>,
    pub redir: Option<Redir<'i>>,
}

either! { Item Command Let Def }
either! { System Path Ident }
either! { Dispatch System Local }
either! { Term Variable Ident Dot DotDot }
either! { Value Term Path Opt Literal }
either! { Opt LongOpt ShortOpt VarShortOpt }
either! { Body Block Command Expr }
either! { Redir Pipe Variable File }
either! { Literal Text }
either! { Expr Literal }

pub fn deq_prepend<T, C, D>(t: T, c: C) -> D
where
    C: IntoIterator,
    C::Item: Into<T>,
    D: std::iter::FromIterator<T>,
{
    std::iter::once(t)
        .chain(c.into_iter().map(<_>::into))
        .collect()
}

pub trait PathOps {
    fn set_abs(self, abs: bool) -> Self;
}
impl<'i> PathOps for Path<'i> {
    fn set_abs(self, abs: bool) -> Self {
        let (_, path) = self;
        (abs, path)
    }
}

impl<'i> LongOpt<'i> {
    pub fn new(s: &'i str) -> Self {
        let s = &s[2..];
        let mut i = s.splitn(2, '=');
        match (i.next(), i.next()) {
            (Some(name), None) => (name, None),
            (Some(name), value) => (name, value),
            other => panic!("{:?}", other),
        }
        .into()
    }
}

impl<'i> ShortOpt<'i> {
    pub fn new(s: &'i str) -> Self {
        let s = &s[1..];
        let name = &s[0..1];
        (if s.len() > 1 {
            (name, Some(&s[1..]))
        } else {
            (name, None)
        })
        .into()
    }
}

impl<'i> VarShortOpt<'i> {
    pub fn new(s: &'i str) -> Self {
        let s = &s[1..];
        let name = &s[1..];
        Self(name)
    }
}
