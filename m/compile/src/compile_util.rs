use super::{i, te, BorrowMut, Compiler, EmitExt, MemTake, Result, SymInfo};

pub trait CompileUtil: BorrowMut<Compiler> {
    fn cmp(&mut self) -> &mut Compiler {
        self.borrow_mut()
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
}

impl CompileUtil for Compiler {}
