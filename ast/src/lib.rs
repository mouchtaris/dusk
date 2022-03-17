pub const VERSION: &str = "0.0.1";

pub type Module<'i> = Any<Item<'i>>;

macro_rules! either {
    ($name:ident $(, $alt:ident)*) => {
        ::either::either! {
            #[derive(Debug)]
            pub $name <'i>
            $(, $alt <'i> )*
        }
    };
}
macro_rules! name {
    ($name:ident $(, $t:ident)*) => {
        ::either::name! {
            #[derive(Debug)]
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

either![Body, Item];
either![Item, Invocation, LetStmt, Empty];
either![
    InvocationTarget,
    InvocationTargetLocal,
    InvocationTargetSystemName,
    InvocationTargetSystemPath
];
either![InvocationArg, Ident, Opt, Path, String, Variable, Word];
either![InvocationRedirection, RedirectInput, RedirectOutput];
either![Path, AbsPath, RelPath, HomePath];
either![Opt, ShortOpt, LongOpt];
either![Redirect, Path, Variable];
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
name![InvocationTargetLocal, Ident];
name![InvocationTargetSystemName, Ident];
name![InvocationTargetSystemPath, Path];
name![
    Invocation,
    AnyDocComment,
    InvocationTarget,
    OptPath,
    AnyInvocationRedirection,
    AnyInvocationEnv,
    AnyInvocationArg
];

pub type LetStmt<'i> = (Ident<'i>, BoxBody<'i>);
pub type InvocationEnv<'i> = (Ident<'i>, InvocationArg<'i>);
pub type Text<'i> = &'i str;
pub type Ident<'i> = Text<'i>;
pub type DocComment<'i> = Text<'i>;
pub type OptText<'i> = Option<Text<'i>>;
pub type OptPath<'i> = Option<Path<'i>>;
pub type AnyDocComment<'i> = Any<DocComment<'i>>;
pub type AnyInvocationArg<'i> = Any<InvocationArg<'i>>;
pub type AnyInvocationEnv<'i> = Any<InvocationEnv<'i>>;
pub type AnyInvocationRedirection<'i> = Any<InvocationRedirection<'i>>;
pub type BoxBody<'i> = Box<Body<'i>>;
pub type Any<T> = Vec<T>;

pub type Empty<'i> = std::marker::PhantomData<&'i ()>;
