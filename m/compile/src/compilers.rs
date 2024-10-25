use super::*;
use ast::*;
use error::ldebug;

type S<T> = CompileEv<T, SymInfo>;

pub trait Compilers<'i> {
    fn path() -> S<Path<'i>> {
        use ast::Path as P;
        |cmp, path| match path {
            P::HomePath(ast::HomePath(p))
            | P::AbsPath(ast::AbsPath(p))
            | P::RelPath(ast::RelPath(p)) => cmp.compile_text(&p.0),
        }
    }
    fn item() -> S<Item<'i>> {
        |cmp, item| match item {
            ast::Item::Expr(e) => {
                let sinfo = te!(cmp.compile(e));
                te!(cmp.emit_cleanup(i::CleanUp, &sinfo));
                Ok(sinfo)
            }
            ast::Item::LetStmt(ast::LetStmt((name, expr))) => {
                let sinfo = te!(cmp.compile(expr));
                ldebug!("type (let) {}: {:?}", name, sinfo);
                cmp.alias_name(name, &sinfo);
                te!(cmp.emit_cleanup(i::Collect, &sinfo));
                Ok(sinfo)
            }
            ast::Item::SrcStmt(ast::SrcStmt((name, expr))) => {
                let sinfo = te!(cmp.compile(expr));
                ldebug!("type (src) {}: {:?}", name, sinfo);
                cmp.alias_name(name, &sinfo);
                te!(cmp.emit_cleanup(i::Pipe, &sinfo));
                Ok(sinfo)
            }
            ast::Item::DefStmt(ast::DefStmt((name, body))) => {
                cmp.emit1(i::Jump { addr: 0 });
                let jump_instr = cmp.instr_id();

                cmp.emit1(i::Allocate { size: 0 });
                let alloc_instr = cmp.instr_id();

                cmp.enter_scope();
                let retval = te!(cmp.compile(body));
                let frame_size = cmp.stack_frame_size();

                te!(cmp.emit_from_symbol(false, &retval));

                cmp.exit_scope();

                te!(cmp.backpatch_with(alloc_instr, frame_size));
                cmp.emit1(i::Return(frame_size));

                let jump_target = cmp.instr_id() + 1;
                te!(cmp.backpatch_with(jump_instr, jump_target));

                let ninfo = cmp.new_address(name, jump_instr + 1, &retval);
                ldebug!("type (def) {}: {:?}", name, ninfo);

                Ok(ninfo)
            }
            ast::Item::Include(ast::Include((path,))) => {
                te!(
                    IncludeExt::include(cmp, path.to_string().as_str()),
                    "Including: {}",
                    path
                );
                Ok(SymInfo::NULL)
            }
            ast::Item::Empty(_) => Ok(SymInfo::NULL),
        }
    }
    fn expr() -> S<Expr<'i>> {
        |cmp, expr| match expr {
            ast::Expr::String(s) => cmp.compile(s),
            ast::Expr::Natural(n) => cmp.compile(n),
            ast::Expr::Invocation(invc) => cmp.compile(invc),
            ast::Expr::Variable(var) => cmp.compile_variable_as_auto(var),
            ast::Expr::Slice(slice) => cmp.compile_slice(slice),
            ast::Expr::Array(closure) => cmp.compile_array(closure),
        }
    }
    fn block() -> S<Block<'i>> {
        |cmp, ast::Block((items, expr))| {
            for item in items {
                te!(cmp.compile(item));
            }
            cmp.compile(expr)
        }
    }
    fn body() -> S<Body<'i>> {
        |cmp, body| match body {
            ast::Body::Block(block) => cmp.compile(block),
        }
    }
    fn module() -> S<Module<'i>> {
        |cmp, ast::Module((body,))| {
            // cmp = te!(cmp.compile(body));

            const MAIN: &str = "m___system_main___";
            const MAIN_CALL: &str = "m___system_main___ $args";

            let body = ast::Body::Block(body);
            let def_stmt = ast::DefStmt((MAIN, body));
            let main_func = ast::Item::DefStmt(def_stmt);

            let invc = te!(facade::parse_invocation(MAIN_CALL));
            let invc = ast::Expr::Invocation(invc);

            let program = ast::Block((vec![main_func], invc));

            // Allocate minimal stack for call tmp local variables
            const CALL_CTX: usize = 8;
            cmp.emit1(i::Allocate { size: CALL_CTX });
            te!(cmp.compile(program));
            cmp.emit1(i::Return(CALL_CTX));
            Ok(SymInfo::NULL)
        }
    }
    fn invocation() -> S<Invocation<'i>> {
        |cmp,
         ast::Invocation((
            _doc_comment_opt,
            invocation_target,
            cwd_opt,
            input_redirections,
            output_redirections,
            envs,
            mut args,
        ))| {
            // TODO
            if !output_redirections.is_empty() {
                error::lwarn!("ignoring output redirections");
            }

            // === Parsings ===
            //
            // Envs
            // ---
            let mut envs_sinfos = vec![];
            for (env_name, env_val) in envs {
                envs_sinfos.push((te!(cmp.compile_text(env_name)), te!(cmp.compile(env_val))))
            }
            //
            // Redirections
            //let inp_redir_len = input_redirections.len();
            //if inp_redir_len > 1 {
            //    temg!("More than one stdin redirections is not (yet) supported");
            //}
            let inp_redir_sinfos = te!(cmp.compile(input_redirections));
            // target
            let invctrgt = format!("{}", invocation_target);
            let invc_target_sinfo = te!(cmp.compile(invocation_target));
            // cwd
            let cwd_sinfo = if let Some(cwd) = cwd_opt {
                te!(cmp.compile(cwd))
            } else {
                SymInfo::NULL
            };
            // args
            args.reverse();
            let args_sinfos = te!(cmp.compile(args));

            // === Emits ===
            // RetVal Allocation
            let mut retval_si = match &invc_target_sinfo {
                SymInfo {
                    typ: sym::Typ::Address(addr),
                    ..
                } => cmp.new_local_tmp(addr.ret_t.as_ref(), ""),
                SymInfo {
                    typ:
                        t @ sym::Typ::Literal(sym::Literal {
                            lit_type: sym::LitType::String | sym::LitType::Syscall,
                            ..
                        }),
                    ..
                } =>
                // Reusing the native type because it's the same allocation size (1)
                // as a Job value (just so as not to introduce a Job type)
                // (so this is a job-type allocation, because all Literal::String
                // and Literal::Syscall invocation targets do return that).
                {
                    cmp.new_local_tmp(SymInfo::typ(t.to_owned()), "")
                }
                other => temg!("What invocation target is this? {:?}", other),
            }
            .clone();
            cmp.emit_allocation(&retval_si);
            // Environment Variables
            for (env_name, env_var) in &envs_sinfos {
                te!(cmp.emit_from_symbol(true, env_var));
                te!(cmp.emit_from_symbol(true, env_name));
            }
            let envs_len = envs_sinfos.len();
            cmp.new_local_tmp(SymInfo::lit_natural(envs_len), "");
            cmp.emit1(i::PushNat(envs_len as usize));
            // Input Redirections
            for inprdi in &inp_redir_sinfos {
                te!(cmp.emit_from_symbol(true, inprdi));
            }
            let inp_redir_len = inp_redir_sinfos.len();
            cmp.new_local_tmp(
                SymInfo::lit_natural(inp_redir_len),
                format_args!("inp_redir_len-{}", invctrgt),
            );
            cmp.emit1(i::PushNat(inp_redir_len as usize));
            // Invocation target
            te!(cmp.emit_from_symbol(true, &invc_target_sinfo));
            // CWD
            te!(cmp.emit_from_symbol(true, &cwd_sinfo));
            // Arguments
            let mut args_len = 0;
            for argi in &args_sinfos {
                args_len += argi.typ.size();
                te!(cmp.emit_from_symbol(true, argi));
            }
            te!(cmp.emit_from_symbol(true, &SymInfo::lit_natural(args_len as usize)));

            const NOWHERE: usize = 0xffffffff;
            match invc_target_sinfo.typ {
                sym::Typ::Address(_) => cmp.emit1(i::Call(NOWHERE)),
                sym::Typ::Literal(sym::Literal {
                    id,
                    lit_type: sym::LitType::Syscall,
                }) => cmp.emit1(i::Syscall(id)),
                sym::Typ::Local(_)
                | sym::Typ::Literal(sym::Literal {
                    lit_type: sym::LitType::String,
                    ..
                }) => cmp.emit1(i::Syscall(vm::syscall::SPAWN)),
                sym::Typ::Literal(_) => {
                    // TODO skip everything above if this is the case
                    retval_si = invc_target_sinfo;
                }
            }

            ldebug!("Retval SI ({:?}): {:?}", invctrgt, retval_si);
            Ok(retval_si)
        }
    }

    fn invocation_cwd() -> S<InvocationCwd<'i>> {
        use InvocationCwd as C;
        |cmp, node| match node {
            C::Path(path) => cmp.compile(path),
            C::Variable(var) => cmp.compile_variable_as_auto(var),
            C::BoxInvocation(invc) => cmp.compile(*invc),
        }
    }

    fn invocation_input_redirection() -> S<RedirectInput<'i>> {
        |cmp, node| match node {
            RedirectInput((Redirect::Path(_path),)) => todo!(),
            RedirectInput((Redirect::Invocation(invc),)) => cmp.compile(invc),
            RedirectInput((Redirect::Variable(var),)) => cmp.compile_variable_as_auto(var),
            RedirectInput((Redirect::Slice(slice),)) => cmp.compile_slice(slice),
            RedirectInput((Redirect::Dereference(_deref),)) => todo!(),
            RedirectInput((Redirect::String(text),)) => cmp.compile(text),
        }
    }

    fn invocation_output_redirection() -> S<RedirectOutput<'i>> {
        |_cmp, node| match node {
            RedirectOutput((Redirect::Path(_path),)) => todo!(),
            RedirectOutput((Redirect::Invocation(_invc),)) => todo!(),
            RedirectOutput((Redirect::Variable(_id),)) => todo!(),
            RedirectOutput((Redirect::Slice(_slice),)) => todo!(),
            RedirectOutput((Redirect::Dereference(_deref),)) => todo!(),
            RedirectOutput((Redirect::String(_text),)) => todo!(),
        }
    }

    fn invocation_option() -> S<Opt<'i>> {
        use ast::Opt as O;
        |cmp, opt| match opt {
            O::LongOpt(ast::LongOpt((a,))) | O::ShortOpt(ast::ShortOpt((a,))) => {
                cmp.compile_text(a)
            }
        }
    }

    fn natural() -> S<Natural<'i>> {
        |cmp, ast::Natural((n,))| cmp.compile_natural(n)
    }

    fn string() -> S<String<'i>> {
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

    fn invocation_arg() -> S<InvocationArg<'i>> {
        |cmp, invocation_argument| {
            use ast::InvocationArg as A;
            match invocation_argument {
                A::Opt(opt) => cmp.compile(opt),
                A::String(s) => cmp.compile(s),
                A::Ident(id) => cmp.compile_text(id),
                A::Variable(ast::Variable(("args",))) => Ok(SymInfo::args()),
                A::Slice(slice) => cmp.compile_slice(slice),
                A::Variable(var) => cmp.compile_variable_as_auto(var),
                A::Path(path) => cmp.compile(path),
                A::Natural(n) => cmp.compile(n),
                A::Invocation(invc) => cmp.compile(invc),
                other => panic!("{:?}", other),
            }
        }
    }

    fn invocation_target() -> S<InvocationTarget<'i>> {
        |cmp, invocation_target| {
            use ast::InvocationTarget as T;

            use T::InvocationTargetLocal as TLocal;
            use T::InvocationTargetSystemName as TSysName;
            use T::InvocationTargetSystemPath as TSysPath;

            use ast::InvocationTargetLocal as Local;
            use ast::InvocationTargetSystemName as SysName;
            use ast::InvocationTargetSystemPath as SysPath;

            Ok(match invocation_target {
                TLocal(Local(("__syscall-argslice",))) => SymInfo::syscall(vm::syscall::ARG_SLICE),
                TLocal(Local(("__builtin",))) => SymInfo::syscall(vm::syscall::BUILTIN),
                TLocal(Local((id,))) => te!(cmp.compile_funcaddr(id)),
                TSysName(SysName((id,))) => te!(cmp.compile_text(id)),
                TSysPath(SysPath((path,))) => te!(cmp.compile(path)),
            })
        }
    }
}

pub struct CompilersImpl;
impl<'i> Compilers<'i> for CompilersImpl {}
