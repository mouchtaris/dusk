use super::{cmps, mem, te, Compiler, Compilers, Result, SymInfo};

pub trait Compile<T>: Sized {
    type RetVal: Default;

    fn compile(&mut self, node: T) -> Result<Self::RetVal>;
}

pub type CompileEv<T, R> = fn(&mut Compiler, T) -> Result<R>;

macro_rules! compile {
    ($($t:ident, $rt:ty, $c:expr),*) => {
        $(
        impl <'i> Compile<ast::$t<'i>> for Compiler {
            type RetVal = $rt;
            fn compile(&mut self, node: ast::$t<'i>) -> Result<$rt> {
                let f: fn(&mut Self, ast::$t<'i>) -> Result<$rt> = $c;
                f(self, node)
            }
        }
        )*
    }
}
compile![
    Natural,
    SymInfo,
    cmps::natural(),
    String,
    SymInfo,
    cmps::string(),
    Path,
    SymInfo,
    cmps::path(),
    Expr,
    SymInfo,
    cmps::expr(),
    InvocationTarget,
    SymInfo,
    cmps::invocation_target(),
    InvocationInputRedirection,
    SymInfo,
    cmps::invocation_input_redirection(),
    InvocationOutputRedirection,
    SymInfo,
    cmps::invocation_output_redirection(),
    InvocationArg,
    SymInfo,
    cmps::invocation_arg(),
    Opt,
    SymInfo,
    cmps::invocation_option(),
    Invocation,
    SymInfo,
    cmps::invocation(),
    Item,
    SymInfo,
    cmps::item(),
    Module,
    SymInfo,
    cmps::module(),
    Block,
    SymInfo,
    cmps::block(),
    Body,
    SymInfo,
    cmps::body()
];

impl<C, N> Compile<Vec<N>> for C
where
    C: Compile<N>,
{
    type RetVal = Vec<C::RetVal>;
    fn compile(&mut self, nodes: Vec<N>) -> Result<Self::RetVal> {
        let cmp = self;
        let mut rv = vec![];
        for node in nodes {
            rv.push(te!(cmp.compile(node)));
        }
        Ok(rv)
    }
}

impl<C, N> Compile<Box<N>> for C
where
    C: Compile<N>,
    N: Default,
{
    type RetVal = C::RetVal;

    fn compile(&mut self, mut node: Box<N>) -> Result<Self::RetVal> {
        let cmp = self;
        let node = mem::take(node.as_mut());
        cmp.compile(node)
    }
}
