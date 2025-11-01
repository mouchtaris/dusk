use super::{iter, Compiler, Mut, Result};

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
        let cmp = self.borrow_mut();

        for i in instr {
            cmp.icode.instructions.push_back(i.into());
        }
    }

    /// Emit the given instructions into instr_table
    fn emit_move<I>(mut self, instr: I) -> Result<Self>
    where
        I: IntoIterator,
        I::Item: Into<vm::Instr>,
        Self: Sized,
    {
        let cmp = self.borrow_mut();

        for i in instr {
            cmp.icode.instructions.push_back(i.into());
        }

        Ok(self)
    }

    /// Emit a single instruction into the instr_table
    fn emit1_move<I>(mut self, instr: I) -> Result<Self>
    where
        I: Into<vm::Instr>,
        Self: Sized,
    {
        let cmp = self.borrow_mut();

        cmp.emit1(instr);

        Ok(self)
    }
}
