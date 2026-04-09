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
            ast::Item::IncludeStr(ast::IncludeStr((ident, path))) => {
                let sinfo = te!(
                    IncludeExt::include_str(cmp, ident, path.to_string().as_str()),
                    "Including as string: {}",
                    path
                );
                Ok(sinfo)
            }
            ast::Item::IncludeBin(ast::IncludeBin((ident, path))) => {
                let sinfo = te!(
                    IncludeExt::include_bin(cmp, ident, path.to_string().as_str()),
                    "Including as bytes: {}",
                    path
                );
                Ok(sinfo)
            }
            ast::Item::TemplateDef(ast::TemplateDef {
                name,
                body,
                const_params,
            }) => {
                const PLACEHOLDER: usize = 0xDEAD_BEEF;

                // Jump over the template body (it's dead code in the main stream)
                cmp.emit1(i::Jump { addr: 0 });
                let jump_instr = cmp.instr_id();

                cmp.emit1(i::Allocate { size: 0 });
                let alloc_instr = cmp.instr_id();
                let body_start = alloc_instr;

                cmp.template_ctx = Some(super::TemplateCompileCtx {
                    body_start,
                    holes: Vec::new(),
                });

                cmp.enter_scope();

                // Register each const param — const_param_idx on the SymInfo
                // drives hole recording at emission time.
                for (idx, cp) in const_params.iter().enumerate() {
                    match cp.typ {
                        ast::ConstParamType::Func => {
                            let si = SymInfo::address(PLACEHOLDER, &SymInfo::NULL)
                                .with_const_param_idx(idx);
                            cmp.insert_to_scope(cp.name.to_string(), si);
                        }
                        ast::ConstParamType::String => {
                            let si = SymInfo::lit_string(PLACEHOLDER)
                                .with_const_param_idx(idx);
                            cmp.alias_name(cp.name, &si);
                        }
                        ast::ConstParamType::Number => {
                            let si = SymInfo::lit_natural(PLACEHOLDER)
                                .with_const_param_idx(idx);
                            cmp.alias_name(cp.name, &si);
                        }
                    }
                }

                let retval = te!(cmp.compile(body));
                let frame_size = cmp.stack_frame_size();

                te!(cmp.emit_from_symbol(false, &retval));

                cmp.exit_scope();

                te!(cmp.backpatch_with(alloc_instr, frame_size));
                cmp.emit1(i::Return(frame_size));

                let jump_target = cmp.instr_id() + 1;
                te!(cmp.backpatch_with(jump_instr, jump_target));

                // Extract template body and holes (recorded at emission time)
                let body_instrs: Vec<vm::Instr> = cmp
                    .icode
                    .instructions
                    .iter()
                    .skip(body_start)
                    .take(jump_target - body_start)
                    .cloned()
                    .collect();

                let holes = cmp.template_ctx.take().unwrap().holes;

                let const_param_kinds: Vec<super::ConstParamKind> = const_params
                    .iter()
                    .map(|cp| match cp.typ {
                        ast::ConstParamType::Func => super::ConstParamKind::Func,
                        ast::ConstParamType::String => super::ConstParamKind::String,
                        ast::ConstParamType::Number => super::ConstParamKind::Number,
                    })
                    .collect();

                let template_id = cmp.templates.len();
                cmp.templates.push(super::TemplateEntry {
                    instructions: body_instrs,
                    holes,
                    const_param_count: const_params.len(),
                    const_param_kinds,
                    ret_t: retval,
                });

                let ninfo = cmp.new_template(name, template_id, const_params.len());
                ldebug!("type (template) {}: {:?}", name, ninfo);

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
            ast::Expr::Variable(var) => cmp.compile_variable_as_auto(var),
            ast::Expr::Slice(slice) => cmp.compile_slice(slice),
            ast::Expr::Array(closure) => cmp.compile_array(closure),
            ast::Expr::AddressOf(ast::AddressOf((name,))) => cmp.compile_funcaddr(name),
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

            // TODO replace with
            // - facade::compile_wrap_in_invocation
            // - facade::compile_invocation

            const MAIN: &str = "m___system_main___";
            const MAIN_CALL: &str = "m___system_main___ $args";

            let body = ast::Body::Block(body);
            let def_stmt = ast::DefStmt((MAIN, body));
            let main_func = ast::Item::DefStmt(def_stmt);

            let invc = te!(facade::parse_invocation(MAIN_CALL));
            let invc = ast::Expr::Invocation(invc);

            let program = ast::Block((vec![main_func], invc));

            // Allocate minimal stack for call tmp local variables
            const CALL_CTX: usize = 7;
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
            let mut invc_target_sinfo = te!(cmp.compile(invocation_target));

            // === Template instantiation ===
            // If the target is a template, instantiate it:
            // - separate const args (AddressOf) from regular args
            // - copy template instructions, patch holes
            // - register as a new concrete function
            if let sym::Typ::Template(ref tmpl) = invc_target_sinfo.typ {
                let template_id = tmpl.template_id;
                let const_param_count = tmpl.const_param_count;

                // Separate const args from regular args
                let tmpl_entry = cmp.templates[template_id].clone();
                let mut const_values: Vec<usize> = Vec::with_capacity(const_param_count);
                let mut regular_args = Vec::new();
                let mut const_idx = 0usize;
                for arg in args.iter() {
                    if let ast::InvocationArg::AddressOf(ast::AddressOf((text,))) = arg {
                        let kind = tmpl_entry.const_param_kinds[const_idx];
                        let val = match kind {
                            super::ConstParamKind::Func => {
                                let addr_si = te!(cmp.compile_funcaddr(text));
                                te!(addr_si.addr())
                            }
                            super::ConstParamKind::String => {
                                // Strip surrounding quotes if present
                                let s = if (text.starts_with('"') && text.ends_with('"'))
                                    || (text.starts_with('\'') && text.ends_with('\''))
                                {
                                    &text[1..text.len() - 1]
                                } else {
                                    text
                                };
                                te!(cmp.add_string(s))
                            }
                            super::ConstParamKind::Number => {
                                te!(text.parse::<usize>())
                            }
                        };
                        const_values.push(val);
                        const_idx += 1;
                    } else {
                        regular_args.push(arg.clone());
                    }
                }

                if const_values.len() != const_param_count {
                    temg!(
                        "Template {} expects {} const args, got {}",
                        invctrgt,
                        const_param_count,
                        const_values.len()
                    );
                }

                // Clone template and patch holes (instruction opcode stays, value changes)
                let mut instrs = tmpl_entry.instructions.clone();
                for &(instr_idx, param_idx) in &tmpl_entry.holes {
                    let val = const_values[param_idx];
                    instrs[instr_idx] = match instrs[instr_idx] {
                        i::PushFuncAddr(_) => i::PushFuncAddr(val),
                        i::RetFuncAddr(_) => i::RetFuncAddr(val),
                        i::PushStr(_) => i::PushStr(val),
                        i::RetStr(_) => i::RetStr(val),
                        i::PushNat(_) => i::PushNat(val),
                        i::RetNat(_) => i::RetNat(val),
                        other => panic!("Unexpected hole instruction: {:?}", other),
                    };
                }

                // Jump over the instantiated body (it's only reached via Call)
                cmp.emit1(i::Jump { addr: 0 });
                let jump_instr = cmp.instr_id();

                // Append instantiated instructions to the main stream
                let inst_addr = cmp.icode.instructions.len();
                for instr in instrs {
                    cmp.emit1(instr);
                }

                // Backpatch the jump to skip over the body
                let after_body = cmp.icode.instructions.len();
                te!(cmp.backpatch_with(jump_instr, after_body));

                // Register as a concrete Address
                invc_target_sinfo = SymInfo::address(inst_addr, &tmpl_entry.ret_t);
                args = regular_args;
            }

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
                } => cmp.new_local_tmp(addr.ret_t.as_ref(), "").to_owned(),
                SymInfo {
                    typ:
                        t @ (sym::Typ::Literal(sym::Literal {
                            lit_type: sym::LitType::String | sym::LitType::Syscall,
                            ..
                        })
                        | sym::Typ::Local(_)),
                    ..
                } =>
                // Reusing the native type because it's the same allocation size (1)
                // as a Job value (just so as not to introduce a Job type)
                // (so this is a job-type allocation, because all Literal::String
                // and Literal::Syscall invocation targets do return that).
                {
                    cmp.new_local_tmp(SymInfo::typ(t.to_owned()), "").to_owned()
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
                sym::Typ::Template(_) => {
                    unreachable!("Template should have been instantiated above")
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
                A::AddressOf(ast::AddressOf((name,))) => cmp.compile_funcaddr(name),
                other => panic!("{:?}", other),
            }
        }
    }

    fn invocation_target() -> S<InvocationTarget<'i>> {
        |cmp, invocation_target| {
            use ast::InvocationTarget as T;

            use T::InvocationTargetDereference as TDeref;
            use T::InvocationTargetInvocation as TInvoc;
            use T::InvocationTargetLocal as TLocal;
            use T::InvocationTargetSystemName as TSysName;
            use T::InvocationTargetSystemPath as TSysPath;

            use ast::InvocationTargetDereference as Deref;
            use ast::InvocationTargetInvocation as Invoc;
            use ast::InvocationTargetLocal as Local;
            use ast::InvocationTargetSystemName as SysName;
            use ast::InvocationTargetSystemPath as SysPath;

            use T::InvocationTargetFuncPtrDeref as TFuncPtr;
            use ast::InvocationTargetFuncPtrDeref as FuncPtr;

            Ok(match invocation_target {
                TLocal(Local(("__syscall-argslice",))) => SymInfo::syscall(vm::syscall::ARG_SLICE),
                TLocal(Local(("__builtin",))) => SymInfo::syscall(vm::syscall::BUILTIN),
                TLocal(Local((id,))) => {
                    let sinfo = te!(cmp.lookup(id));
                    match &sinfo.typ {
                        sym::Typ::Template(_) => sinfo.to_owned(),
                        _ => te!(cmp.compile_funcaddr(id)),
                    }
                }
                TSysName(SysName((id,))) => te!(cmp.compile_text(id)),
                TSysPath(SysPath((path,))) => te!(cmp.compile(path)),
                TDeref(Deref((Dereference((name,)),))) => {
                    let sinfo = te!(cmp.compile(ast::let_stmt(name, ast::Variable((name,)))));
                    te!(cmp.emit_cleanup(i::BufferString, &sinfo));
                    sinfo
                }
                TInvoc(Invoc((inv,))) => {
                    let inv_si = te!(cmp.compile(*inv));
                    te!(cmp.emit_cleanup(i::BufferString, &inv_si));
                    inv_si
                }
                TFuncPtr(FuncPtr((ast::Variable((name,)),))) => {
                    te!(cmp.compile_funcaddr(name))
                }
            })
        }
    }
}

pub struct CompilersImpl;
impl<'i> Compilers<'i> for CompilersImpl {}
