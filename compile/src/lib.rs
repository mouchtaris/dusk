pub const VERSION: &str = "0.0.1";
use collection::{Entry, Map};
use std::{io, mem};
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
    retval: SymInfo,
}

pub type Scope = Map<String, SymInfo>;

#[derive(Debug, Default, Clone)]
pub struct SymInfo {
    pub id: usize,
}

pub trait Compile<T>: Sized {
    fn compile(self, node: &T) -> Result<Self>;
}

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
    Module,
    |mut cmp, module| {
        cmp.enter_scope();
        cmp.emit1(i::Allocate { size: 0 });
        let instr_alloc = cmp.instr_id();

        for item in module {
            match item {
                ast::Item::Invocation(invc) => {
                    cmp = te!(cmp.compile(invc));
                    soft_todo!();
                }
            }
        }

        let frame_size = cmp.stack_frame_size();
        te!(cmp.backpatch(instr_alloc, |i| match i {
            i::Allocate { size } => Ok(*size = frame_size),
            _ => terr!("not an allocate instr"),
        }));

        Ok(cmp)
    },
    Invocation,
    |mut cmp: Compiler,
     ast::Invocation((doc_comment_opt, invocation_target, cwd_opt, redirections, envs, args))| {
        cmp = te!(cmp.compile(invocation_target));
        let jobid = cmp.retval.id;

        if let Some(path) = cwd_opt {
            cmp.retval = cmp.new_tmp_var();
            te!(cmp.path_to_string(path));

            let cwdid = cmp.retval.id;
            cmp.emit1(i::JobSetCwd { jobid, cwdid });
        }

        soft_todo!();
        let _ = redirections;
        soft_todo!();
        let _ = envs;
        {
            cmp.retval = cmp.new_tmp_var();
            let argid = cmp.retval.id;
            for arg in args {
                cmp = te!(cmp.compile(arg));
                cmp.emit1(i::JobPushArg { jobid, argid });
            }
        }

        let argc_var = cmp.new_tmp_var();
        cmp.emit([i::CompleteProcessJob { jobid }]);

        Ok(cmp)
    },
    InvocationArg,
    |mut cmp, invocation_argument| {
        use ast::InvocationArg as A;
        match invocation_argument {
            A::Opt(opt) => cmp.opt_to_string(opt),
            A::String(s) => cmp.string_to_string(s),
            A::Ident(id) => cmp.text_to_string(id),
            other => panic!("{:?}", other),
        }
        .map(|_| cmp)
    },
    InvocationTarget,
    |mut cmp, invocation_target| {
        let mut var_path = cmp.new_tmp_var();
        let var_proc = cmp.new_tmp_var();

        use ast::InvocationTarget as T;

        match invocation_target {
            &T::InvocationTargetLocal(ast::InvocationTargetLocal((_id,))) => {
                todo!()
            }
            &T::InvocationTargetSystemName(ast::InvocationTargetSystemName((id,))) => {
                let strid = te!(cmp.add_string(id));
                let var = cmp.new_tmp_var();
                cmp.emit([
                    i::LoadString { strid, dst: var.id },
                    i::FindInBinPath {
                        id: var.id,
                        dst: var_path.id,
                    },
                ]);
            }
            ast::InvocationTarget::InvocationTargetSystemPath(ast::InvocationTargetSystemPath(
                (path,),
            )) => {
                mem::swap(&mut cmp.retval, &mut var_path);
                te!(cmp.path_to_string(path));
                mem::swap(&mut cmp.retval, &mut var_path);
            }
        }
        cmp.emit([i::CreateProcessJob {
            path: var_path.id,
            dst: var_proc.id,
        }]);
        cmp.retval = var_proc;
        Ok(cmp)
    }
];

impl Compiler {
    pub fn init(&mut self) {
        self.add_string("").unwrap();
    }

    pub fn compile<N>(self, node: &N) -> Result<Self>
    where
        Self: Compile<N>,
    {
        Compile::compile(self, node)
    }

    pub fn write_to<O>(&self, o: io::Result<O>) -> io::Result<()>
    where
        O: io::Write,
    {
        o.and_then(|mut o| {
            Ok({
                writeln!(o, "=== STRINGS ===")?;
                for (s, vm::StringInfo { id }) in &self.icode.strings {
                    writeln!(o, "[{}] {:?}", id, s)?;
                }
                writeln!(o, "=== ICODE ===")?;
                for instr in &self.icode.instructions {
                    writeln!(o, "{:?}", instr)?;
                }
                writeln!(o, "=== SYMBOLS ===")?;
                for (scope_id, scope) in &self.sym_info {
                    writeln!(o, "-- SCOPE {}", scope_id)?;
                    for (name, sym_info) in scope {
                        writeln!(o, ": {:12} : {:?}", name, sym_info)?;
                    }
                }
            })
        })
    }

    fn add_string<S>(&mut self, s: S) -> Result<usize>
    where
        S: Into<String>,
    {
        use vm::StringInfo as SI;
        let SI { id } = te!(SI::add(&mut self.icode, s));
        Ok(id)
    }

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
        match scope.entry(name) {
            Entry::Occupied(occ) => panic!("Tmp variable re-entry: {}", occ.key()),
            Entry::Vacant(vac) => vac.insert(sym_info).clone(),
        }
    }

    /// [`text_to_string`] this raw string
    fn string_to_string(&mut self, ast::String((s,)): &ast::String) -> Result<()> {
        let cmp = self;
        cmp.text_to_string(&s[1..s.len() - 1])
    }

    /// [`text_to_string`] this option
    fn opt_to_string(&mut self, opt: &ast::Opt) -> Result<()> {
        let cmp = self;
        use ast::Opt as O;
        match opt {
            O::LongOpt(ast::LongOpt((a,))) | O::ShortOpt(ast::ShortOpt((a,))) => {
                cmp.text_to_string(a)
            }
        }
    }

    /// [`text_to_string`] this path
    fn path_to_string(&mut self, path: &ast::Path) -> Result<()> {
        let cmp = self;
        use ast::Path as P;
        match path {
            P::HomePath(ast::HomePath(p))
            | P::AbsPath(ast::AbsPath(p))
            | P::RelPath(ast::RelPath(p)) => cmp.text_to_string(&p.0),
        }
    }

    /// Add the given string as a literal and emit to load it into `cmp.retval`
    fn text_to_string<S>(&mut self, text: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let cmp = self;
        let text = text.as_ref();
        let strid = te!(cmp.add_string(text));
        let dst = cmp.retval.id;
        cmp.emit1(i::LoadString { strid, dst });
        Ok(())
    }
}
