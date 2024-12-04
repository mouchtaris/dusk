pub const VERSION: &str = "0.0.1";

mod display;
mod factory;
pub use factory::*;

macro_rules! either {
    ($name:ident $(, $alt:ident)*) => {
        ::either::either! {
            #[derive(Debug, Clone)]
            pub $name <'i>
            $(, $alt <'i> )*
        }
    };
}
macro_rules! name {
    ($name:ident $(, $t:ident)*) => {
        ::either::name! {
            #[derive(Debug, Clone)]
            pub $name <'i> = (
                $($t <'i> , )*
            )
        }
    };
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name;
    };
}

name![Module, Block];

either![Item, Expr, LetStmt, DefStmt, SrcStmt, Include, Empty];
either![Expr, Invocation, String, Natural, Slice, Variable, Array];
either![Body, Block];
either![
    InvocationTarget,
    InvocationTargetLocal,
    InvocationTargetSystemName,
    InvocationTargetSystemPath,
    InvocationTargetDereference,
    InvocationTargetInvocation
];
either![
    InvocationArg,
    Ident,
    Opt,
    Path,
    String,
    Variable,
    Word,
    Natural,
    Invocation,
    Slice
];
either![InvocationCwd, Path, Variable, BoxInvocation];
either![Path, AbsPath, RelPath, HomePath];
either![Opt, ShortOpt, LongOpt];
either![
    Redirect,
    Path,
    Variable,
    Dereference,
    Invocation,
    Slice,
    String
];
either![Range, DoubleRange, Index];

name![Array, AnyExpr];
name![Include, Path];
name![Block, AnyItem, Expr];
name![LetStmt, Ident, Expr];
name![SrcStmt, Ident, Expr];
name![DefStmt, Ident, Body];
name![RedirectInput, Redirect];
name![RedirectOutput, Redirect];
name![String, Text];
name![Word, Text];
name![AbsPath, Text];
name![RelPath, Text];
name![HomePath, Text];
name![LongOpt, Text];
name![ShortOpt, Text];
name![Variable, Text];
name![Slice, Text, BoxRange];
name![Dereference, Text];
name![Natural, Text];
name![InvocationTargetLocal, Ident];
name![InvocationTargetSystemName, Ident];
name![InvocationTargetSystemPath, Path];
name![InvocationTargetDereference, Dereference];
name![InvocationTargetInvocation, BoxInvocation];
name![
    Invocation,
    AnyDocComment,
    InvocationTarget,
    OptInvocationCwd,
    AnyInvocationInputRedirection,
    AnyInvocationOutputRedirection,
    AnyInvocationEnv,
    AnyInvocationArg
];

pub type InvocationInputRedirection<'i> = RedirectInput<'i>;
pub type InvocationOutputRedirection<'i> = RedirectOutput<'i>;
pub type InvocationEnv<'i> = (Ident<'i>, InvocationArg<'i>);
pub type Text<'i> = &'i str;
pub type Ident<'i> = Text<'i>;
pub type DocComment<'i> = Text<'i>;
pub type OptText<'i> = Option<Text<'i>>;
pub type OptPath<'i> = Option<Path<'i>>;
pub type OptInvocationCwd<'i> = Option<InvocationCwd<'i>>;
pub type Index<'i> = InvocationArg<'i>;
pub type DoubleRange<'i> = Tupl2<Index<'i>>;
pub type AnyDocComment<'i> = Any<DocComment<'i>>;
pub type AnyInvocationArg<'i> = Any<InvocationArg<'i>>;
pub type AnyInvocationEnv<'i> = Any<InvocationEnv<'i>>;
pub type AnyInvocationInputRedirection<'i> = Any<InvocationInputRedirection<'i>>;
pub type AnyInvocationOutputRedirection<'i> = Any<InvocationOutputRedirection<'i>>;
pub type AnyItem<'i> = Any<Item<'i>>;
pub type AnyExpr<'i> = Any<Expr<'i>>;
pub type BoxBody<'i> = Box<Body<'i>>;
pub type BoxInvocation<'i> = Box<Invocation<'i>>;
pub type BoxRange<'i> = Box<Range<'i>>;
pub type Any<T> = Vec<T>;

pub type Empty<'i> = std::marker::PhantomData<&'i ()>;
pub type Tupl2<T> = (T, T);
