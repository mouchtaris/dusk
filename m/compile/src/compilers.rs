use super::*;
use ast::*;

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
                cmp.alias_name(name, &sinfo);
                te!(cmp.emit_cleanup(i::Collect, &sinfo));
                Ok(sinfo)
            }
            ast::Item::SrcStmt(ast::SrcStmt((name, expr))) => {
                let sinfo = te!(cmp.compile(expr));
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

                let ninfo = cmp.new_address(name, jump_instr + 1);

                Ok(ninfo)
            }
            ast::Item::Empty(_) => Ok(SymInfo::NULL),
        }
    }
    fn expr() -> S<Expr<'i>> {
        |cmp, expr| match expr {
            ast::Expr::String(s) => cmp.compile(s),
            ast::Expr::Natural(n) => cmp.compile(n),
            ast::Expr::Invocation(invc) => cmp.compile(invc),
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

            let body = ast::Body::Block(body);
            let def_stmt = ast::DefStmt((MAIN, body));
            let main_func = ast::Item::DefStmt(def_stmt);

            let invc = te!(facade::parse_invocation(MAIN));
            let invc = ast::Expr::Invocation(invc);

            let program = ast::Block((vec![main_func], invc));

            // Allocate minimal stack for call tmp local variables
            const CALL_CTX: usize = 5;
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
            let _ = envs;
            let _ = output_redirections;

            // === Parsings ===
            //
            // Redirections
            let inp_redir_len = input_redirections.len();
            let _inp_redir_sinfos = te!(cmp.compile(input_redirections));
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
            let arg_len = args.len();
            args.reverse();
            let args_sinfos = te!(cmp.compile(args));

            // === Emits ===
            //
            // RetVal allocation
            let mut retval = cmp
                .new_local_tmp(format_args!("retval-{}", invctrgt))
                .clone();
            cmp.emit1(i::PushNull); // retval allocation
                                    // argn
            cmp.new_local_tmp(format_args!("inp_redir_len-{}", invctrgt));
            cmp.emit1(i::PushNat(inp_redir_len));
            te!(cmp.emit_from_symbol(true, &invc_target_sinfo)); // target
            te!(cmp.emit_from_symbol(true, &cwd_sinfo)); // cwd
            for argi in &args_sinfos {
                // args
                error::ltrace!("arg sinfo: {:?}", argi);
                te!(cmp.emit_from_symbol(true, argi));
            }
            cmp.new_local_tmp(format_args!("argc-{}", invctrgt));
            cmp.emit1(i::PushNat(arg_len)); // argn

            const NOWHERE: usize = 0xffffffff;
            match invc_target_sinfo.typ {
                sym::Typ::Address(_) => cmp.emit1(i::Call(NOWHERE)),
                sym::Typ::Local(_)
                | sym::Typ::Literal(sym::Literal {
                    lit_type: sym::LitType::String,
                    ..
                }) => cmp.emit1(i::Spawn(NOWHERE)),
                sym::Typ::Literal(_) => {
                    // TODO skip everything above if this is the case
                    retval = invc_target_sinfo;
                }
            }

            Ok(retval)
        }
    }

    fn invocation_input_redirection() -> S<RedirectInput<'i>> {
        |cmp, node| match node {
            RedirectInput((Redirect::Path(_path),)) => todo!(),
            RedirectInput((Redirect::Variable(ast::Variable((var,))),)) => {
                let sinfo = te!(cmp.lookup(var)).to_owned();
                te!(cmp.emit_from_symbol(true, &sinfo));
                Ok(sinfo)
            }
            RedirectInput((Redirect::Dereference(_deref),)) => todo!(),
        }
    }

    fn invocation_output_redirection() -> S<RedirectOutput<'i>> {
        |_cmp, node| match node {
            RedirectOutput((Redirect::Path(_path),)) => todo!(),
            RedirectOutput((Redirect::Variable(_id),)) => todo!(),
            RedirectOutput((Redirect::Dereference(_deref),)) => todo!(),
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
                A::Variable(ast::Variable(("args",))) => {
                    let sinfo = cmp.new_local_tmp("args_for_callee").clone();
                    cmp.emit1(i::PushArgs);
                    Ok(sinfo)
                }
                A::Variable(ast::Variable((var,))) => match te!(cmp.lookup(var)) {
                    sinfo @ SymInfo {
                        typ: sym::Typ::Literal(_),
                        ..
                    } => Ok(sinfo.to_owned()),
                    sinfo @ SymInfo {
                        typ: sym::Typ::Local(_),
                        scope_id,
                    } => {
                        if *scope_id == cmp.scope_id() {
                            Ok(sinfo.to_owned())
                        } else {
                            temg!(
                                "{} is in scope {} instead of {}",
                                var,
                                scope_id,
                                cmp.scope_id()
                            )
                        }
                    }
                    _sinfo @ &SymInfo {
                        typ: sym::Typ::Address(_),
                        ..
                    } => {
                        //Ok(temg!("Not supported yet"))
                        let name = te!(cmp.lookup_name(_sinfo)).to_owned();
                        let letstmt = ast::let_stmt(&name, ast::invoc(&name));
                        let local_si: SymInfo = te!(cmp.compile(letstmt)).to_owned();
                        error::ldebug!("capture call to {} in {:?}", name, local_si);
                        error::ldebug!("new {}: {:?}", name, te!(cmp.lookup(name.as_str())));

                        Ok(local_si)
                    }
                },
                A::Path(path) => cmp.compile(path),
                A::Natural(n) => cmp.compile(n),
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
                TLocal(Local((id,))) => te!(cmp.compile_funcaddr(id)),
                TSysName(SysName((id,))) => te!(cmp.compile_text(id)),
                TSysPath(SysPath((path,))) => te!(cmp.compile(path)),
            })
        }
    }
}

pub struct CompilersImpl;
impl<'i> Compilers<'i> for CompilersImpl {}
