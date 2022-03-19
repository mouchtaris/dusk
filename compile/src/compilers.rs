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
    fn box_body() -> E<BoxBody<'i>> {
        |cmp, body| match body.as_ref() {
            ast::Body::Item(item) => cmp.compile(item),
        }
    }
    fn item() -> E<Item<'i>> {
        |mut cmp, item| match item {
            ast::Item::Invocation(invc) => {
                cmp = te!(cmp.compile(invc));
                cmp.emit1(i::SysCall(vm::syscall::CREATE_PROCESS_JOB));
                Ok(cmp)
            }
            ast::Item::LetStmt(ast::LetStmt((_name, body))) => {
                cmp = te!(cmp.compile(body));
                soft_todo!();
                Ok(cmp)
            }
            ast::Item::DefStmt(ast::DefStmt((name, body))) => {
                cmp.enter_scope();
                cmp.emit1(i::Jump { addr: 0 });
                let jump_instr = cmp.instr_id();

                cmp = te!(cmp.compile(body));
                cmp.exit_scope();

                let jump_target = cmp.instr_id() + 1;
                te!(cmp.backpatch(jump_instr, |i| {
                    Ok(match i {
                        i::Jump { addr } => *addr = jump_target,
                        _ => terr!("not a jump instr"),
                    })
                }));

                let _ = name;
                soft_todo!();
                Ok(cmp)
            }
            ast::Item::Empty(_) => Ok(cmp),
        }
    }
    fn module() -> E<Module<'i>> {
        |mut cmp, ast::Module((items,))| {
            cmp.enter_scope();

            cmp.emit1(i::Allocate { size: 0 });
            let alloc_instr = cmp.instr_id();

            for item in items {
                cmp = te!(cmp.compile(item));
            }

            let frame_size = cmp.stack_frame_size();
            te!(cmp.backpatch(alloc_instr, |i| Ok(te!(i.allocate_size(frame_size)))));

            cmp.exit_scope();

            Ok(cmp)
        }
    }
    fn invocation() -> E<Invocation<'i>> {
        |mut cmp: Compiler,
         ast::Invocation((
            doc_comment_opt,
            invocation_target,
            cwd_opt,
            redirections,
            envs,
            args,
        ))| {
            cmp = te!(cmp.compile(invocation_target));
            // job_type
            cmp.new_local_tmp("process-job-type");
            cmp.emit1(i::PushNat(cmp.retval));

            cmp = te!(cmp.compile(cwd_opt));
            // if let Some(path) = cwd_opt {
            //     cmp.retval = cmp.new_tmp_var();
            //     te!(cmp.path_to_string(path));

            //     let cwdid = cmp.retval.id;
            //     cmp.emit1(i::JobSetCwd { jobid, cwdid });
            // }

            soft_todo!();
            let _ = redirections;
            soft_todo!();
            let _ = envs;

            let cmp = te!(cmp.compile(args));

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

            cmp = te!(match invocation_target {
                &TLocal(Local((_id,))) => todo!(),
                &TSysName(SysName((id,))) => {
                    cmp.retval = PROCESS_JOB_TYPE;
                    cmp.compile_text(id)
                }
                TSysPath(SysPath((path,))) => {
                    cmp.retval = PROCESS_JOB_TYPE;
                    cmp.compile(path)
                }
            });

            Ok(cmp)
        }
    }
}

pub struct CompilersImpl;
impl<'i> Compilers<'i> for CompilersImpl {}

pub const PROCESS_JOB_TYPE: usize = 0x00;
