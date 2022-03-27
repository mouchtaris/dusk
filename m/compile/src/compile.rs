use super::{cmps, i, mem, te, Compiler, Compilers, EmitExt, Result, SymInfo, SymbolTableExt};

pub trait Compile<T>: Sized {
    type RetVal: Default;

    fn compile_eval(self, node: T, rv: &mut Self::RetVal) -> Result<Self>;

    fn eval(self, node: T) -> Result<(Self, Self::RetVal)> {
        let mut rv = <_>::default();
        let mut cmp = self;

        cmp = te!(cmp.compile_eval(node, &mut rv));

        Ok((cmp, rv))
    }

    fn compile(self, node: T) -> Result<Self> {
        self.compile_eval(node, &mut <_>::default())
    }
}

pub type CompileEv<T> = fn(Compiler, T) -> Result<Compiler>;
pub type EvalEv<T, R> = fn(&mut Compiler, T) -> Result<R>;

pub fn from_compile<T>(cmp: &mut Compiler, cev: CompileEv<T>, node: T) -> Result<SymInfo> {
    *cmp = te!(cev(mem::take(cmp), node));
    Ok(cmp.retval.clone())
}
macro_rules! eval {
    ($($t:ident, $rt:ty, $c:expr),*) => {
        $(
        impl <'i> Compile<ast::$t<'i>> for Compiler {
            type RetVal = $rt;
            fn compile_eval(mut self, node: ast::$t<'i>, retval: &mut $rt) -> Result<Self> {
                let f: fn(&mut Self, ast::$t<'i>) -> Result<$rt> = $c;
                *retval = te!(f(&mut self, node));
                Ok(self)
            }
        }
        )*
    }
}
macro_rules! compile {
    ($($t:ident, $c:expr),*) => {
        $(
        impl <'i> Compile<ast::$t<'i>> for Compiler {
            type RetVal = ();
            fn compile_eval(self, node: ast::$t<'i>, _: &mut ()) -> Result<Self> {
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
    InvocationInputRedirection,
    cmps::invocation_input_redirection(),
    InvocationOutputRedirection,
    cmps::invocation_output_redirection(),
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
    Variable,
    cmps::variable(),
    Dereference,
    cmps::dereference()
];
eval![Natural, SymInfo, cmps::natural()];

impl<C, N> Compile<Option<N>> for C
where
    C: Compile<N> + EmitExt + SymbolTableExt,
{
    type RetVal = Option<C::RetVal>;
    fn compile_eval(self, node: Option<N>, rv: &mut Self::RetVal) -> Result<Self> {
        let mut cmp = self;

        match node {
            Some(node) => {
                let mut retval = <_>::default();
                cmp = te!(cmp.compile_eval(node, &mut retval));
                *rv = Some(retval);
                Ok(cmp)
            }
            None => {
                *rv = None;
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
    type RetVal = Vec<C::RetVal>;
    fn compile_eval(self, nodes: Vec<N>, rv: &mut Self::RetVal) -> Result<Self> {
        let mut cmp = self;
        for node in nodes {
            let mut v = <_>::default();
            cmp = te!(cmp.compile_eval(node, &mut v));
            rv.push(v);
        }
        Ok(cmp)
    }
}

impl<C, N> Compile<Box<N>> for C
where
    C: Compile<N>,
    N: Default,
{
    type RetVal = C::RetVal;

    fn compile_eval(self, mut node: Box<N>, rv: &mut Self::RetVal) -> Result<Self> {
        let cmp = self;
        let node = mem::take(node.as_mut());
        cmp.compile_eval(node, rv)
    }
}
