use super::{cmps, i, te, Compiler, Compilers, EmitExt, Result, SymbolTableExt};

pub trait Compile<T>: Sized {
    fn compile(self, node: &T) -> Result<Self>;
}

pub type CompileEv<T> = fn(Compiler, &T) -> Result<Compiler>;

macro_rules! compile {
    ($($t:ident, $c:expr),*) => {
        $(
        impl <'i> Compile<ast::$t<'i>> for Compiler {
            fn compile(self, node: &ast::$t<'i>) -> Result<Self> {
                let f: fn(Self, &ast::$t<'i>) -> Result<Self> = $c;
                f(self, node)
            }
        }
        )*
    }
}
compile![
    BoxBody,
    cmps::box_body(),
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
    cmps::string()
];

impl<C, N> Compile<Option<N>> for C
where
    C: Compile<N> + EmitExt + SymbolTableExt,
{
    fn compile(self, node: &Option<N>) -> Result<Self> {
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
    fn compile(self, nodes: &Vec<N>) -> Result<Self> {
        let mut cmp = self;
        let len = nodes.len();
        for i in 0..len {
            let node = &nodes[len - 1 - i];
            cmp = te!(cmp.compile(node));
        }
        Ok(cmp)
    }
}
