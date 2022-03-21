use super::CompileEv as E;
use super::*;
use ast::*;

pub trait Compilers<'i> {
    fn path() -> E<Path<'i>> {
        use ast::Path as P;
        |cmp, path| match path {
            P::HomePath(ast::HomePath(p))
            | P::AbsPath(ast::AbsPath(p))
            | P::RelPath(ast::RelPath(p)) => cmp.compile_text(&p.0),
        }
    }
    fn item() -> E<Item<'i>> {
        |mut cmp, item| match item {
            ast::Item::Invocation(invc) => {
                cmp = te!(cmp.compile(invc));
                Ok(cmp)
            }
            ast::Item::LetStmt(ast::LetStmt((_name, body))) => {
                cmp = te!(cmp.compile(body));
                soft_todo!();
                Ok(cmp)
            }
            ast::Item::DefStmt(ast::DefStmt((name, body))) => {
                cmp.emit1(i::Jump { addr: 0 });
                let jump_instr = cmp.instr_id();

                cmp = te!(cmp.compile(body));

                cmp.emit1(i::Return);

                let jump_target = cmp.instr_id() + 1;
                te!(cmp.backpatch_with(jump_instr, jump_target));

                cmp.new_address(*name, jump_instr + 1);

                soft_todo!();
                Ok(cmp)
            }
            ast::Item::Empty(_) => Ok(cmp),
        }
    }
    fn block() -> E<Block<'i>> {
        |mut cmp, ast::Block((items,))| {
            cmp.enter_scope();

            cmp.emit1(i::Allocate { size: 0 });
            let alloc_instr = cmp.instr_id();

            for item in items {
                cmp = te!(cmp.compile(item));
            }

            let frame_size = cmp.stack_frame_size();
            cmp.emit1(i::Dealloc(frame_size));
            te!(cmp.backpatch_with(alloc_instr, frame_size));

            cmp.exit_scope();

            Ok(cmp)
        }
    }
    fn body() -> E<Body<'i>> {
        |cmp, body| match body {
            ast::Body::Block(block) => cmp.compile(block),
        }
    }
    fn module() -> E<Module<'i>> {
        |cmp, ast::Module((body,))| cmp.compile(body)
    }
    fn invocation() -> E<Invocation<'i>> {
        |mut cmp: Compiler,
         ast::Invocation((
            _doc_comment_opt,
            invocation_target,
            cwd_opt,
            redirections,
            envs,
            args,
        ))| {
            soft_todo!();
            let _ = redirections;

            soft_todo!();
            let _ = envs;

            cmp = te!(cmp.compile(args));
            cmp.new_local_tmp("argc");
            cmp.emit1(i::PushNat(args.len()));

            cmp = te!(cmp.compile(cwd_opt));

            // job_type
            cmp = te!(cmp.compile(invocation_target));
            let job_type = cmp.retval;
            cmp.emit1(i::PushNat(job_type));
            cmp.new_local_tmp("process-job-type");

            cmp.emit1(i::SysCall(vm::syscall::CREATE_JOB));

            Ok(cmp)
        }
    }

    fn invocation_option() -> E<Opt<'i>> {
        use ast::Opt as O;
        |cmp, opt| match opt {
            O::LongOpt(ast::LongOpt((a,))) | O::ShortOpt(ast::ShortOpt((a,))) => {
                cmp.compile_text(a)
            }
        }
    }

    fn string() -> E<String<'i>> {
        |cmp, ast::String((s,))| cmp.compile_text(&s[1..s.len() - 1])
    }

    fn invocation_arg() -> E<InvocationArg<'i>> {
        |cmp, invocation_argument| {
            use ast::InvocationArg as A;
            match invocation_argument {
                A::Opt(opt) => cmp.compile(opt),
                A::String(s) => cmp.compile(s),
                A::Ident(id) => cmp.compile_text(id),
                A::Variable(ast::Variable((_name,))) => {
                    todo!()
                }
                A::Path(path) => cmp.compile(path),
                other => panic!("{:?}", other),
            }
        }
    }

    fn invocation_target() -> E<InvocationTarget<'i>> {
        |mut cmp, invocation_target| {
            use ast::InvocationTarget as T;

            use T::InvocationTargetLocal as TLocal;
            use T::InvocationTargetSystemName as TSysName;
            use T::InvocationTargetSystemPath as TSysPath;

            use ast::InvocationTargetLocal as Local;
            use ast::InvocationTargetSystemName as SysName;
            use ast::InvocationTargetSystemPath as SysPath;

            cmp = match invocation_target {
                &TLocal(Local((id,))) => {
                    let addr = te!(cmp.lookup_addr(id), "Missing variable: {}", id).addr;

                    cmp.new_local_tmp("fun_invc_trg_addr");
                    cmp.emit1(i::PushNat(addr));

                    cmp.retval = FUNCTION_JOB_TYPE;
                    cmp
                }
                &TSysName(SysName((id,))) => {
                    cmp = te!(cmp.compile_text(id));
                    cmp.retval = PROCESS_JOB_TYPE;
                    cmp
                }
                TSysPath(SysPath((path,))) => {
                    cmp = te!(cmp.compile(path));
                    cmp.retval = PROCESS_JOB_TYPE;
                    cmp
                }
            };

            Ok(cmp)
        }
    }
}

pub struct CompilersImpl;
impl<'i> Compilers<'i> for CompilersImpl {}

pub const PROCESS_JOB_TYPE: usize = 0x00;
pub const FUNCTION_JOB_TYPE: usize = 0x01;
