use super::{
    i, sym, te, temg, Borrow, BorrowMut, Compiler, EmitExt, MemTake, Result, SymInfo,
    SymbolTableExt,
};

pub trait CompileUtil: Borrow<Compiler> + BorrowMut<Compiler> {
    fn cmp(&mut self) -> &mut Compiler {
        self.borrow_mut()
    }
    fn cmp_ref(&self) -> &Compiler {
        self.borrow()
    }

    fn emit_cleanup<S>(&mut self, style: S) -> Result<SymInfo>
    where
        S: FnOnce(usize) -> i,
    {
        let cmp = self.cmp();

        let retval_info = cmp.retval.mem_take();
        let fp_off = te!(retval_info.fp_off());

        cmp.emit1(style(fp_off));

        Ok(retval_info)
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
}

impl CompileUtil for Compiler {}
