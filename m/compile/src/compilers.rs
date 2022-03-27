use super::CompileEv as E;
use super::*;
use ast::*;

type S<T> = EvalEv<T, SymInfo>;

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
                te!(cmp.emit_cleanup(i::CleanUp));
                Ok(cmp)
            }
            ast::Item::LetStmt(ast::LetStmt((name, expr))) => {
                cmp = te!(cmp.compile(expr));
                let retval = te!(cmp.emit_cleanup(i::Collect));
                cmp.alias_name(name, &retval);
                Ok(cmp)
            }
            ast::Item::SrcStmt(ast::SrcStmt((name, expr))) => {
                cmp = te!(cmp.compile(expr));
                let retval = te!(cmp.emit_cleanup(i::Pipe));
                cmp.alias_name(name, &retval);
                Ok(cmp)
            }
            ast::Item::DefStmt(ast::DefStmt((name, body))) => {
                cmp.emit1(i::Jump { addr: 0 });
                let jump_instr = cmp.instr_id();

                cmp.emit1(i::Allocate { size: 0 });
                let alloc_instr = cmp.instr_id();

                cmp.enter_scope();
                let (mut cmp, _tmps) = te!(cmp.eval(body));
                //cmp = te!(cmp.compile(body));
                let frame_size = cmp.stack_frame_size();

                let retval = te!(cmp.retval.fp_off());
                cmp.emit1(i::SetRetVal(retval));

                cmp.exit_scope();

                te!(cmp.backpatch_with(alloc_instr, frame_size));
                cmp.emit1(i::Return(frame_size));

                let jump_target = cmp.instr_id() + 1;
                te!(cmp.backpatch_with(jump_instr, jump_target));

                cmp.new_address(name, jump_instr + 1);

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
            for item in items {
                cmp = te!(cmp.compile(item));
            }
            cmp = te!(cmp.compile(expr));

            Ok(cmp)
        }
    }
    fn body() -> E<Body<'i>> {
        |cmp, body| match body {
            ast::Body::Block(block) => cmp.compile(block),
        }
    }
    fn module() -> E<Module<'i>> {
        |mut cmp, ast::Module((body,))| {
            // cmp = te!(cmp.compile(body));

            const MAIN: &str = "m___system_main___";

            let body = ast::Body::Block(body);
            let def_stmt = ast::DefStmt((MAIN, body));
            let main_func = ast::Item::DefStmt(def_stmt);

            let invc = te!(facade::parse_invocation(MAIN));
            let invc = ast::Expr::Invocation(invc);

            let program = ast::Block((vec![main_func], invc));

            // Allocate minimal stack for call tmp local variables
            const CALL_CTX: usize = 5;
            cmp.emit1(i::Allocate { size: CALL_CTX });
            cmp = te!(cmp.compile(program));
            cmp.emit1(i::Return(CALL_CTX));
            Ok(cmp)
        }
    }
    fn invocation() -> E<Invocation<'i>> {
        |mut cmp: Compiler,
         ast::Invocation((
            _doc_comment_opt,
            invocation_target,
            cwd_opt,
            input_redirections,
            output_redirections,
            envs,
            mut args,
        ))| {
            let retval = cmp.new_local_tmp("retval").clone();
            cmp.emit1(i::PushNull);

            // Redirections
            let len = input_redirections.len();
            cmp = te!(cmp.compile(input_redirections));
            cmp.new_local_tmp("inp_redir_len");
            cmp.emit1(i::PushNat(len));

            // TODO
            let _ = envs;
            let _ = output_redirections;

            // target
            cmp = te!(cmp.compile(invocation_target));
            let job_type = cmp.retval.val();

            // cwd
            cmp = te!(cmp.compile(cwd_opt));

            // args
            let arg_len = args.len();
            args.reverse();
            cmp = te!(cmp.compile(args));
            cmp.new_local_tmp("argc");

            // argn
            cmp.emit1(i::PushNat(arg_len));

            const NOWHERE: usize = 0xffffffff;
            match job_type {
                PROCESS_JOB_TYPE => cmp.emit1(i::Spawn(NOWHERE)),
                FUNCTION_JOB_TYPE => cmp.emit1(i::Call(NOWHERE)),
                other => panic!("{:?}", other),
            }

            cmp.retval = retval;
            Ok(cmp)
        }
    }

    fn invocation_input_redirection() -> E<RedirectInput<'i>> {
        |cmp, node| match node {
            RedirectInput((Redirect::Path(_path),)) => todo!(),
            RedirectInput((Redirect::Variable(var),)) => cmp.compile(var),
            RedirectInput((Redirect::Dereference(deref),)) => cmp.compile(deref),
        }
    }

    fn invocation_output_redirection() -> E<RedirectOutput<'i>> {
        |_cmp, node| match node {
            RedirectOutput((Redirect::Path(_path),)) => todo!(),
            RedirectOutput((Redirect::Variable(_id),)) => todo!(),
            RedirectOutput((Redirect::Dereference(_deref),)) => todo!(),
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

    fn natural() -> S<Natural<'i>> {
        |cmp, ast::Natural((n,))| compile::from_compile(cmp, Compiler::compile_natural, n)
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

    fn variable() -> E<Variable<'i>> {
        |mut cmp, Variable((name,))| {
            let fp_off = te!(cmp.lookup_local_var(name)).fp_off;
            cmp.new_local_tmp(format!("copy {}", name));
            cmp.emit1(i::PushLocal(fp_off));
            Ok(cmp)
        }
    }

    fn dereference() -> E<Dereference<'i>> {
        |mut cmp, Dereference((name,))| {
            let addr = te!(cmp.lookup_addr(name)).addr;
            cmp.new_local_tmp(format!("copy {}", name));
            cmp.emit1(i::PushFuncAddr(addr));
            Ok(cmp)
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
                A::Variable(var) => cmp.compile(var),
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
                TLocal(Local((id,))) => {
                    cmp = te!(cmp.compile_funcaddr(id));
                    cmp.retval = SymInfo::just(FUNCTION_JOB_TYPE);
                    cmp
                }
                TSysName(SysName((id,))) => {
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
