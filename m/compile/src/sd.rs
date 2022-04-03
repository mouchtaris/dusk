use super::{
    sym::{Address, LitType, Literal, Local, Typ},
    te, Compiler, SymInfo, SymbolTable,
};

buf::sd_struct![SymbolTable, scopes, scope_stack];

buf::sd_struct![SymInfo, typ, scope_id];

buf::sd_type![Typ, Local, 0u8, Address, 1u8, Literal, 2u8];

buf::sd_struct![Local, fp_off, is_alias];
buf::sd_struct![Address, addr];
buf::sd_struct![Literal, lit_type, id];

buf::sd_enum![LitType, Null, 0u8, String, 1u8, Natural, 2u8];

buf::sd![
    Compiler,
    |Compiler { icode, sym_table }, mut dst| {
        icode.write_to(Ok(&mut dst))?;
        sym_table.write_out(&mut dst)?;
        Ok(())
    },
    |mut inp| {
        Ok(Compiler {
            icode: te!(
                vm::ICode::load_from(Ok(&mut inp)).map_err(|e| format!("Loading icode: {:?}", e))
            ),
            sym_table: te!(<_>::read_in(&mut inp)),
        })
    }
];
