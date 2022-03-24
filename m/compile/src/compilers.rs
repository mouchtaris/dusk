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
            ast::Item::Expr(e) => {
                cmp = te!(cmp.compile(e));
                cmp.emit1(i::CleanUp(te!(cmp.retval.fp_off())));
                Ok(cmp)
            }
            ast::Item::LetStmt(ast::LetStmt((name, expr))) => {
                cmp = te!(cmp.compile(expr));
                let val_info = cmp.retval.mem_take();
                cmp.alias_name(*name, &val_info);
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

                Ok(cmp)
            }
            ast::Item::Empty(_) => Ok(cmp),
        }
    }
    fn expr() -> E<Expr<'i>> {
        |cmp, expr| match expr {
            ast::Expr::String(s) => cmp.compile(s),
            ast::Expr::Natural(n) => cmp.compile(n),
            ast::Expr::Invocation(invc) => cmp.compile(invc),
        }
    }
    fn block() -> E<Block<'i>> {
        |mut cmp, ast::Block((items, expr))| {
            cmp.enter_scope();

            cmp.emit1(i::Allocate { size: 0 });
            let alloc_instr = cmp.instr_id();

            for item in items {
                cmp = te!(cmp.compile(item));
            }
            cmp = te!(cmp.compile(expr));

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
            let retval = cmp.new_local_tmp("retval").clone();
            cmp.emit1(i::PushNull);

            // TODO
            let _ = redirections;
            let _ = envs;

            // target
            cmp = te!(cmp.compile(invocation_target));
            let job_type = cmp.retval.val();
            let job_target = cmp.retval1.val();

            // cwd
            cmp = te!(cmp.compile(cwd_opt));

            // args
            cmp = te!(cmp.compile(args));
            cmp.new_local_tmp("argc");

            // argn
            cmp.emit1(i::PushNat(args.len()));

            match job_type {
                PROCESS_JOB_TYPE => cmp.emit1(i::SysCall(vm::syscall::CREATE_JOB)),
                FUNCTION_JOB_TYPE => cmp.emit1(i::Call(job_target)),
                other => panic!("{:?}", other),
            }

            cmp.retval = retval;
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

    fn natural() -> E<Natural<'i>> {
        |cmp, &ast::Natural((n,))| cmp.compile_natural(n)
    }

    fn string() -> E<String<'i>> {
        |cmp, ast::String((s,))| {
            let t = if s.starts_with('r') {
                let h = s[1..]
                    .chars()
                    .scan(Some(()), |s, c| {
                        s.and_then(|u| match c {
                            '#' => Some(u),
                            _ => None,
                        })
                    })
                    .count();
                &s[1 + h + 1..s.len() - 1 - h]
            } else if s.starts_with('"') {
                &s[1..s.len() - 1]
            } else {
                s
            };
            cmp.compile_text(t)
        }
    }

    fn invocation_arg() -> E<InvocationArg<'i>> {
        |mut cmp, invocation_argument| {
            use ast::InvocationArg as A;
            match invocation_argument {
                A::Opt(opt) => cmp.compile(opt),
                A::String(s) => cmp.compile(s),
                A::Ident(id) => cmp.compile_text(id),
                A::Variable(ast::Variable(("args",))) => {
                    cmp.new_local_tmp("args_for_callee");
                    cmp.emit1(i::PushArgs);
                    Ok(cmp)
                }
                &A::Variable(ast::Variable((name,))) => {
                    let sinfo = te!(cmp.lookup(name));
                    if sinfo.scope_id != cmp.scope_id() {
                        temg!(
                            "{} is in different scope {} than {}",
                            name,
                            sinfo.scope_id,
                            cmp.scope_id()
                        )
                    }
                    let sinfo = te!(sinfo.as_local_ref(), "{}", name);
                    let fp_off = sinfo.fp_off;
                    cmp.new_local_tmp(format!("copy {}", name));
                    cmp.emit1(i::PushLocal(fp_off));
                    Ok(cmp)
                }
                A::Path(path) => cmp.compile(path),
                A::Natural(n) => cmp.compile(n),
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

                    cmp.retval = SymInfo::just(FUNCTION_JOB_TYPE);
                    cmp.retval1 = SymInfo::just(addr);
                    cmp
                }
                &TSysName(SysName((id,))) => {
                    cmp = te!(cmp.compile_text(id));
                    cmp.retval = SymInfo::just(PROCESS_JOB_TYPE);
                    cmp
                }
                TSysPath(SysPath((path,))) => {
                    cmp = te!(cmp.compile(path));
                    cmp.retval = SymInfo::just(PROCESS_JOB_TYPE);
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
