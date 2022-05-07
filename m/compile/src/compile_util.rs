use super::{
    facade, i, sym, te, temg, Borrow, BorrowMut, Compiler, EmitExt, Result, SymInfo, SymbolTableExt,
};

pub trait CompileUtil: Borrow<Compiler> + BorrowMut<Compiler> {
    fn cmp(&mut self) -> &mut Compiler {
        self.borrow_mut()
    }
    fn cmp_ref(&self) -> &Compiler {
        self.borrow()
    }

    fn compile_array(&mut self, ast::Array((items,)): ast::Array) -> Result<SymInfo> {
        let cmp = self.cmp();

        let mut types = Vec::new();
        for item in items {
            types.push(te!(cmp.compile(item)));
        }

        let sinfo = cmp.new_array(types, "an_array");

        Ok(sinfo)
    }

    fn compile_slice(&mut self, ast::Slice((name, br)): ast::Slice) -> Result<SymInfo> {
        let cmp = self.cmp();
        let ast = facade::parse_invocation(
            r###"
                        __syscall-argslice 0 0 $args
                    "###,
        );
        let mut ast = te!(ast);
        let args = &mut (ast.0).6;
        let source = ast::InvocationArg::Variable(ast::Variable((name,)));
        args.clear();
        args.push(br.0);
        args.push(br.1);
        args.push(source);
        cmp.compile(ast)
    }

    fn compile_variable_as_auto(
        &mut self,
        ast::Variable((var,)): ast::Variable,
    ) -> Result<SymInfo> {
        let cmp = self.cmp();
        match te!(cmp.lookup(var)) {
            sinfo @ SymInfo {
                typ: sym::Typ::Literal(_),
                ..
            } => Ok(sinfo.to_owned()),
            sinfo @ SymInfo {
                typ: sym::Typ::Local(_),
                ..
            } => cmp.ensure_local_scope(var, sinfo),
            &SymInfo {
                typ: sym::Typ::Address(_),
                ..
            } => cmp.capture_call_to_local_var(var),
        }
    }

    fn capture_call_to_local_var(&mut self, name: &str) -> Result<SymInfo> {
        let cmp = self.cmp();

        let letstmt = ast::src_stmt(&name, ast::invoc(&name));
        let local_si: SymInfo = te!(cmp.compile(letstmt)).to_owned();
        error::ldebug!("capture call to {} in {:?}", name, local_si);
        error::ldebug!("new {}: {:?}", name, te!(cmp.lookup(name)));

        Ok(local_si)
    }

    fn ensure_local_scope(&self, var: &str, sinfo: &SymInfo) -> Result<SymInfo> {
        let cmp = self.cmp_ref();
        let SymInfo { scope_id, .. } = sinfo;
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

    fn emit_allocation(&mut self, sinfo: &SymInfo) {
        self.cmp().emit((0..sinfo.typ.size()).map(|_| i::PushNull))
    }

    fn emit_from_symbol(&mut self, push_or_retval: bool, sinfo: &SymInfo) -> Result<()> {
        let cmp = self.cmp();

        error::ldebug!(
            "emit from symbol [{}]: {:?}",
            if push_or_retval { "push" } else { "ret" },
            sinfo
        );
        let sinfo = sinfo.to_owned();
        Ok(match &sinfo {
            &SymInfo {
                typ: sym::Typ::Address(sym::Address { addr, .. }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(sinfo, format_args!("func-addr-{}", addr));
                    i::PushFuncAddr
                } else {
                    i::RetFuncAddr
                };
                cmp.emit1(instr(addr))
            }
            &SymInfo {
                typ:
                    sym::Typ::Literal(sym::Literal {
                        id,
                        lit_type: sym::LitType::String,
                    }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(sinfo, format_args!("string-lit-{}", id));
                    i::PushStr
                } else {
                    i::RetStr
                };
                cmp.emit1(instr(id))
            }
            &SymInfo {
                typ:
                    sym::Typ::Literal(sym::Literal {
                        id,
                        lit_type: sym::LitType::Natural,
                    }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(sinfo, format_args!("nat-lit-{}", id));
                    i::PushNat
                } else {
                    i::RetNat
                };

                cmp.emit1(instr(id))
            }
            &SymInfo {
                typ:
                    sym::Typ::Literal(sym::Literal {
                        lit_type: sym::LitType::Null | sym::LitType::Syscall,
                        ..
                    }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(sinfo, format_args!("null"));
                    i::PushNull
                } else {
                    panic!("Return null not supported")
                };

                cmp.emit1(instr)
            }
            &SymInfo {
                typ:
                    sym::Typ::Literal(sym::Literal {
                        lit_type: sym::LitType::Args,
                        ..
                    }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(sinfo, format_args!("args"));
                    i::PushArgs
                } else {
                    return temg!("Cannot return $args");
                };

                cmp.emit1(instr);
            }
            SymInfo {
                typ:
                    sym::Typ::Local(
                        local @ sym::Local {
                            is_alias: false,
                            fp_off,
                            types,
                        },
                    ),
                scope_id,
            } => {
                let size = local.size();
                let instr = |p| {
                    (if push_or_retval {
                        let i = p - fp_off;
                        cmp.new_local_tmp(
                            types[i..(i + 1)].iter().cloned(),
                            format_args!(
                                "copy-of-{}[{}/{}] in {}",
                                fp_off,
                                fp_off + size as usize - p,
                                size,
                                scope_id
                            ),
                        );
                        i::PushLocal
                    } else {
                        i::RetLocal
                    })(p)
                };

                let instr: Vec<_> = local.foreach(instr).collect();
                cmp.emit(instr);
            }
            SymInfo {
                typ:
                    sym::Typ::Local(sym::Local {
                        is_alias: true,
                        types,
                        ..
                    }),
                ..
            } => {
                for typ in types.iter().rev() {
                    te!(cmp.emit_from_symbol(push_or_retval, typ));
                }
            }
        })
    }

    fn emit_cleanup<C>(&mut self, clns: C, sinfo: &SymInfo) -> Result<()>
    where
        C: FnMut(usize) -> i,
    {
        let cmp = self.cmp();

        Ok(match sinfo {
            &SymInfo {
                typ:
                    sym::Typ::Local(
                        ref local @ sym::Local {
                            is_alias: false, ..
                        },
                    ),
                ..
            } => cmp.emit(local.foreach(clns)),
            &SymInfo {
                typ: sym::Typ::Literal(_) | sym::Typ::Local(sym::Local { is_alias: true, .. }),
                ..
            } => (),
            other => panic!("{:?}", other),
        })
    }
}

impl CompileUtil for Compiler {}
