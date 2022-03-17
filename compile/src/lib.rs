pub const VERSION: &str = "0.0.1";
use collection::Map;
use vm::Instr as i;

error::Error! {
    Msg = &'static str
    Message = String
    Vm = vm::Error
}
use error::{soft_todo, te, terr};

#[derive(Debug, Default)]
pub struct Compiler {
    pub icode: vm::ICode,
    sym_info: Map<usize, Scope>,
    current_scope: usize,
}

pub type Scope = Map<String, SymInfo>;

#[derive(Debug, Default, Copy, Eq, Ord, Hash, Clone, PartialEq, PartialOrd)]
pub struct SymInfo {
    pub id: usize,
}

pub trait Compile<T> {
    fn compile(&mut self, node: &T) -> Result<()>;
}

macro_rules! compile {
    ($($t:ident, $c:expr),*) => {
        $(
        impl <'i> Compile<ast::$t<'i>> for Compiler {
            fn compile(&mut self, node: &ast::$t<'i>) -> Result<()> {
                let f: fn(&mut Self, &ast::$t<'i>) -> Result<()> = $c;
                f(self, node)
            }
        }
        )*
    }
}
compile![
    Module,
    |cmp, module| {
        cmp.enter_scope();
        cmp.emit1(i::Allocate { size: 0 });
        let instr_alloc = cmp.instr_id();

        for item in module {
            match item {
                ast::Item::Invocation(invc) => {
                    te!(cmp.compile(invc));
                    soft_todo!();
                }
            }
        }

        let frame_size = cmp.stack_frame_size();
        te!(cmp.backpatch(instr_alloc, |i| match i {
            i::Allocate { size } => Ok(*size = frame_size),
            _ => terr!("not an allocate instr"),
        }));
        Ok(())
    },
    Invocation,
    |cmp,
     ast::Invocation((doc_comment_opt, invocation_target, cwd_opt, redirections, envs, args))| {
        te!(cmp.compile(invocation_target));

        soft_todo!();
        //if let Some(cwd) = cwd_opt {
        //    te!(cmp.compile_path(cwd));
        //    cmp.emit1(i::JobSetCwd);
        //    soft_todo!();
        //}
        soft_todo!();
        let _ = redirections;
        soft_todo!();
        let _ = envs;
        soft_todo!();
        let _ = args;
        cmp.emit1(i::CompleteProcessJob);
        Ok(())
    },
    InvocationTarget,
    |cmp, invocation_target| {
        use ast::InvocationTarget as T;
        match invocation_target {
            &T::InvocationTargetLocal(ast::InvocationTargetLocal((_id,))) => {
                todo!()
            }
            &T::InvocationTargetSystemName(ast::InvocationTargetSystemName((id,))) => {
                let strid = te!(cmp.add_string(id));
                let var = cmp.new_tmp_var();
                let var_path = cmp.new_tmp_var();
                let var_proc = cmp.new_tmp_var();
                cmp.emit([
                    i::LoadString { strid, dst: var.id },
                    i::FindInBinPath {
                        id: var.id,
                        dst: var_path.id,
                    },
                    i::CreateProcessJob {
                        path: var_path.id,
                        dst: var_proc.id,
                    },
                ]);
            }
            ast::InvocationTarget::InvocationTargetSystemPath(_) => todo!(),
        }
        Ok(())
    }
];

impl Compiler {
    pub fn init(&mut self) {
        self.add_string("").unwrap();
    }

    pub fn compile<N>(&mut self, node: &N) -> Result<()>
    where
        Self: Compile<N>,
    {
        Compile::compile(self, node)
    }
    fn add_string<S>(&mut self, s: S) -> Result<usize>
    where
        S: Into<String>,
    {
        use vm::StringInfo as SI;
        let SI { id } = te!(SI::add(&mut self.icode, s));
        Ok(id)
    }
    //
    //    fn compile_path(&mut self, path: &ast::Path) -> Result<()> {
    //        use ast::Path as P;
    //        match path {
    //            P::HomePath(ast::HomePath(p))
    //            | P::AbsPath(ast::AbsPath(p))
    //            | P::RelPath(ast::RelPath(p)) => {
    //                te!(self.compile_string(p.0));
    //            }
    //        }
    //        Ok(())
    //    }
    //
    //    fn compile_string<S>(&mut self, s: S) -> Result<()>
    //    where
    //        S: Into<String>,
    //    {
    //        let strid = te!(self.add_string(s));
    //        self.emit(vec![i::LoadString { strid }]);
    //        Ok(())
    //    }

    fn emit<I>(&mut self, instr: I)
    where
        I: IntoIterator,
        I::Item: Into<vm::Instr>,
    {
        for i in instr {
            self.icode.instructions.push_back(i.into());
        }
    }
    fn emit1<I>(&mut self, instr: I)
    where
        I: Into<vm::Instr>,
    {
        self.emit(std::iter::once(instr.into()))
    }
    fn instr_id(&self) -> usize {
        self.icode.instructions.len() - 1
    }

    fn enter_scope(&mut self) {
        let scope_id = self.current_scope + 1;
        self.sym_info.insert(scope_id, <_>::default());
        self.current_scope = scope_id;
    }

    fn stack_frame_size(&self) -> usize {
        let scope = self.sym_info.get(&self.current_scope).unwrap();
        scope.len()
    }

    fn backpatch<B>(&mut self, instr_id: usize, block: B) -> Result<()>
    where
        B: FnOnce(&mut i) -> Result<()>,
    {
        let instr = te!(self.icode.instructions.get_mut(instr_id));
        block(instr)
    }

    fn new_tmp_var(&mut self) -> SymInfo {
        let scope = self.sym_info.get_mut(&self.current_scope).unwrap();
        let id = scope.len();
        let name = format!("t:{}:{}", self.current_scope, id);
        let sym_info = SymInfo { id };
        scope.insert(name, sym_info);
        sym_info
    }
}
