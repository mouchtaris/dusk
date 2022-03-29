use super::{
    i, sym, te, temg, Borrow, BorrowMut, Compiler, EmitExt, Result, SymInfo, SymbolTableExt,
};

pub trait CompileUtil: Borrow<Compiler> + BorrowMut<Compiler> {
    fn cmp(&mut self) -> &mut Compiler {
        self.borrow_mut()
    }
    fn cmp_ref(&self) -> &Compiler {
        self.borrow()
    }

    fn lookup_local_var<S>(&self, name: S) -> Result<&sym::Local>
    where
        S: AsRef<str>,
    {
        let cmp = self.cmp_ref();
        let name = name.as_ref();

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
        Ok(sinfo)
    }

    fn emit_from_symbol(&mut self, push_or_retval: bool, sinfo: &SymInfo) -> Result<()> {
        let cmp = self.cmp();

        Ok(match sinfo {
            &SymInfo {
                typ: sym::Typ::Address(sym::Address { addr }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(format_args!("func-addr-{}", addr));
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
                    cmp.new_local_tmp(format_args!("string-lit-{}", id));
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
                    cmp.new_local_tmp(format_args!("nat-lit-{}", id));
                    i::PushNat
                } else {
                    i::RetNat
                };

                cmp.emit1(instr(id))
            }
            &SymInfo {
                typ:
                    sym::Typ::Literal(sym::Literal {
                        lit_type: sym::LitType::Null,
                        ..
                    }),
                ..
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(format_args!("null-lit"));
                    i::PushNull
                } else {
                    panic!("Return null not supported")
                };

                cmp.emit1(instr)
            }
            &SymInfo {
                typ: sym::Typ::Local(sym::Local { fp_off, is_alias }),
                scope_id,
            } => {
                let instr = if push_or_retval {
                    cmp.new_local_tmp(format_args!(
                        "copy-of-{} {}in {}",
                        fp_off,
                        if is_alias { "(alias) " } else { "" },
                        scope_id
                    ));
                    i::PushLocal
                } else {
                    i::RetLocal
                };

                cmp.emit1(instr(fp_off))
            }
        })
    }
    fn emit_cleanup<C>(&mut self, clns: C, sinfo: &SymInfo) -> Result<()>
    where
        C: FnOnce(usize) -> i,
    {
        let cmp = self.cmp();

        Ok(match sinfo {
            &SymInfo {
                typ:
                    sym::Typ::Local(sym::Local {
                        fp_off,
                        is_alias: false,
                    }),
                ..
            } => cmp.emit1(clns(fp_off)),
            &SymInfo {
                typ: sym::Typ::Literal(_),
                ..
            } => (),
            other => panic!("{:?}", other),
        })
    }
}

impl CompileUtil for Compiler {}
