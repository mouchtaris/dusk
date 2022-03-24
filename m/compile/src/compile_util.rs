use super::{i, te, BorrowMut, Compiler, EmitExt, MemTake, Result, SymInfo};

pub trait CompileUtil: BorrowMut<Compiler> {
    fn cmp(&mut self) -> &mut Compiler {
        self.borrow_mut()
    }

    fn emit_cleanup2<S>(&mut self, style: S) -> Result<SymInfo>
    where
        S: FnOnce(usize) -> i,
    {
        let cmp = self.cmp();

        let retval_info = cmp.retval.mem_take();
        let fp_off = te!(retval_info.fp_off());

        cmp.emit1(style(fp_off));

        Ok(retval_info)
    }

    fn emit_cleanup_collect(&mut self) -> Result<SymInfo> {
        self.emit_cleanup2(i::CleanUpCollect)
    }
    fn emit_cleanup(&mut self) -> Result<SymInfo> {
        self.emit_cleanup2(i::CleanUp)
    }
}

impl CompileUtil for Compiler {}
