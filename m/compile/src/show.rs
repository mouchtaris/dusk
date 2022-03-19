use super::{io, Compiler, Show, SymbolTable};

impl Show for Compiler {
    fn write_to_impl<O>(&self, mut o: O) -> io::Result<()>
    where
        O: io::Write,
    {
        Ok({
            writeln!(o, "=== STRINGS ===")?;
            for (s, vm::StringInfo { id }) in &self.icode.strings {
                writeln!(o, "[{}] {:?}", id, s)?;
            }
            writeln!(o, "=== ICODE ===")?;
            let mut i = 0;
            for instr in &self.icode.instructions {
                writeln!(o, "[{:4}] {:?}", i, instr)?;
                i += 1;
            }
            writeln!(o, "=== SYMBOLS ===")?;
            self.sym_table.write_to_impl(o)?;
        })
    }
}

impl Show for SymbolTable {
    fn write_to_impl<O>(&self, mut o: O) -> io::Result<()>
    where
        O: io::Write,
    {
        Ok({
            let mut scope_id = 0;
            for scope in &self.scopes {
                writeln!(o, "-- SCOPE {}", scope_id)?;
                scope_id += 1;
                for (name, sym_info) in scope {
                    writeln!(o, ": {:12} : {:?}", name, sym_info)?;
                }
            }
        })
    }
}
