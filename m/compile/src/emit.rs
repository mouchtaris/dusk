use super::{iter, Compiler, Mut, Result, VmICodeMut};

pub trait EmitExt
where
    Self: Mut<Compiler>,
{
    /// Emit a single instruction into the instr_table
    fn emit1<I>(&mut self, instr: I)
    where
        I: Into<vm::Instr>,
    {
        let cmp = self.borrow_mut();
        cmp.emit(iter::once(instr.into()))
    }

    /// Emit the given instructions into instr_table
    fn emit<I>(&mut self, instr: I)
    where
        I: IntoIterator,
        I::Item: Into<vm::Instr>,
    {
        VmICodeMut::emit(&mut self.borrow_mut().icode, instr);
    }

    /// Emit the given instructions into instr_table
    fn emit_move<I>(mut self, instr: I) -> Result<Self>
    where
        I: IntoIterator,
        I::Item: Into<vm::Instr>,
        Self: Sized,
    {
        Self::emit(&mut self, instr);
        Ok(self)
    }

    /// Emit a single instruction into the instr_table
    fn emit1_move<I>(mut self, instr: I) -> Result<Self>
    where
        I: Into<vm::Instr>,
        Self: Sized,
    {
        Self::emit1(&mut self, instr);
        Ok(self)
    }
}
