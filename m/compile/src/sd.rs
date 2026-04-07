use super::{
    sym::{Address, LitType, Literal, Local, Template, Typ},
    te, Compiler, ConstParamKind, SymInfo, TemplateEntry,
};

buf::sd_struct![SymInfo, typ, scope_id];

buf::sd_type![Typ, Local, 0u8, Address, 1u8, Literal, 2u8, Template, 3u8];

buf::sd_struct![Local, fp_off, is_alias, types];
buf::sd_struct![Address, addr, ret_t];
buf::sd_struct![Literal, lit_type, id];
buf::sd_struct![Template, template_id, const_param_count];
buf::sd_struct![TemplateEntry, instructions, holes, const_param_count, const_param_kinds, ret_t];
buf::sd_enum![ConstParamKind, Func, 0u8, String, 1u8, Number, 2u8];

buf::sd_enum![LitType, Null, 0u8, String, 1u8, Natural, 2u8, Syscall, 3u8, Args, 4u8];

buf::sd![
    Compiler,
    |Compiler {
         icode, sym_table, templates, ..
     },
     mut dst| {
        icode.write_to(Ok(&mut dst))?;
        sym_table.write_out(&mut dst)?;
        templates.write_out(&mut dst)?;
        Ok(())
    },
    |mut inp| {
        Ok(Compiler {
            icode: te!(
                vm::ICode::load_from(Ok(&mut inp)).map_err(|e| format!("Loading icode: {:?}", e))
            ),
            sym_table: te!(<_>::read_in(&mut inp)),
            templates: te!(<_>::read_in(&mut inp)),
            current_file_path: <_>::default(),
            template_ctx: None,
        })
    }
];
