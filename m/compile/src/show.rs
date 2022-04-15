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

                let mut buffer: Vec<(String, String)> = scope
                    .iter()
                    .map(|(name, sym_info)| {
                        let name = format!("{:?}", name);
                        let sym_info = format!("{:?}", sym_info);
                        (name, sym_info)
                    })
                    .collect();
                buffer.sort_by(|a, b| a.0.cmp(&b.0));
                let len = buffer
                    .iter()
                    .map(|(n, _)| n.len())
                    .chain(std::iter::once(8))
                    .max()
                    .unwrap_or(0);

                for (name, sym_info) in buffer {
                    writeln!(
                        o,
                        ": {name:len$} : {si}",
                        len = len,
                        name = name,
                        si = sym_info,
                    )?;
                }
            }
        })
    }
}
