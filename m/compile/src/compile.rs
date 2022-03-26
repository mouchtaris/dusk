use super::{cmps, i, mem, te, Compiler, Compilers, EmitExt, Result, SymbolTableExt};

pub trait Compile<T>: Sized {
    fn compile(self, node: T) -> Result<Self>;
}

pub type CompileEv<T> = fn(Compiler, T) -> Result<Compiler>;

macro_rules! compile {
    ($($t:ident, $c:expr),*) => {
        $(
        impl <'i> Compile<ast::$t<'i>> for Compiler {
            fn compile(self, node: ast::$t<'i>) -> Result<Self> {
                let f: fn(Self, ast::$t<'i>) -> Result<Self> = $c;
                f(self, node)
            }
        }
        )*
    }
}
compile![
    Item,
    cmps::item(),
    Module,
    cmps::module(),
    Invocation,
    cmps::invocation(),
    InvocationArg,
    cmps::invocation_arg(),
    InvocationTarget,
    cmps::invocation_target(),
    Path,
    cmps::path(),
    Opt,
    cmps::invocation_option(),
    String,
    cmps::string(),
    Block,
    cmps::block(),
    Body,
    cmps::body(),
    Expr,
    cmps::expr(),
    Natural,
    cmps::natural()
];

impl<C, N> Compile<Option<N>> for C
where
    C: Compile<N> + EmitExt + SymbolTableExt,
{
    fn compile(self, node: Option<N>) -> Result<Self> {
        let mut cmp = self;

        match node {
            Some(n) => cmp.compile(n),
            None => {
                cmp.new_local_tmp("optional-node-null");
                cmp.emit1_move(i::PushNull)
            }
        }
    }
}

impl<C, N> Compile<Vec<N>> for C
where
    C: Compile<N>,
{
    fn compile(self, nodes: Vec<N>) -> Result<Self> {
        let mut cmp = self;
        for node in nodes {
            cmp = te!(cmp.compile(node));
        }
        Ok(cmp)
    }
}

impl<C, N> Compile<Box<N>> for C
where
    C: Compile<N>,
    N: Default,
{
    fn compile(self, mut node: Box<N>) -> Result<Self> {
        let mut cmp = self;
        let node = mem::take(node.as_mut());
        cmp = te!(cmp.compile(node));
        Ok(cmp)
    }
}
