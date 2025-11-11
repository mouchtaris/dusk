use super::*;

pub struct Info {
    pub number_of_imported_global_methods: usize,
    pub number_of_imported_strings: usize,
    pub number_of_imported_instructions: usize,
}

pub fn link_modules<C: Borrow<Compiler>>(mods: impl IntoIterator<Item = C>) -> Compiler {
    let mut cmp = Compiler::new();
    cmp.enter_scope();
    cmp.enter_scope();
    mods.into_iter().fold(cmp, |mut cmp, module| {
        import(&mut cmp, module.borrow());
        cmp
    })
}

pub fn import<T, S>(mut target: T, source: &S) -> Info
where
    T: BorrowMut<Compiler>,
    S: Borrow<Compiler>,
{
    let source: &Compiler = source.borrow();

    // Num of instructions. Used later, need to unborrow target.
    let instrs = target.num_instrs();
    // ID->String hashmap on source. Used later.
    let sstrings: HashMap<usize, &str> = source.strings();

    let Compiler {
        icode, sym_table, ..
    } = target.borrow_mut();
    let vm::ICode {
        instructions,
        strings,
        ..
    } = icode;

    let mut number_of_imported_strings = 0;
    let mut on_add_string = |_: &str| number_of_imported_strings += 1;

    // ---------------------------------------
    // ---- Importing/Translating strings ----
    // ---------------------------------------
    //
    // Translate a given string ID:
    // - If the string exists, return the ID,
    // - else, insert the string and return the ID.
    //
    let mut translate_string_id = |id: usize| -> usize {
        let sstring: &str = sstrings.get(&id).expect("existing string id");
        match get_string_id(strings, sstring) {
            Some(id) => id,
            None => {
                on_add_string(sstring);
                add_string(strings, sstring)
            }
        }
    };

    // -------------------------------
    // ---- Translating func_addr ----
    // -------------------------------
    //
    // This is simpler, just add the number of instructions
    // on the target compiler.
    let translate_addr = |old_addr: usize| -> usize { old_addr + instrs };

    // -------------------------------
    // ---- Translate instruction ----
    // -------------------------------
    // - translate string-ids for string-related instructions
    // - translate addresses for address-related instructions
    let translate_instr = |instr: &vm::Instr| -> vm::Instr {
        use vm::Instr::*;
        match *instr {
            PushStr(id) => PushStr(translate_string_id(id)),
            RetStr(id) => RetStr(translate_string_id(id)),
            PushFuncAddr(addr) => PushFuncAddr(translate_addr(addr)),
            RetFuncAddr(addr) => RetFuncAddr(translate_addr(addr)),
            Jump { addr } => Jump {
                addr: translate_addr(addr),
            },
            _ => *instr,
        }
    };

    // --------------------------------
    // ---- Importing instructions ----
    // --------------------------------
    //
    // Copy all instructions as-is, except:
    // - translate string-ids for string-related instructions,
    // - translate addresses for address-related instructions.
    //
    // NOTE: this process is mutable, besides this direct emit:
    // Side effects:
    // - Strings are imported
    let number_of_imported_instructions = instrs_emit(
        instructions,
        source.icode.instructions.iter().map(translate_instr),
    );

    // -----------------------------------------------
    // ---- Import global address symbols (def-s) ----
    // -----------------------------------------------
    //
    // Insert symbols, only if they are global and refer to func-addr.
    //
    // Translate the function address in the process.
    //
    for (name, sym_id) in source
        .global_scope_opt()
        .expect("global scope in source compiler")
        .symbols()
    {
        let info = sym_id.sym_info();

        if let Ok(sym::Address { addr, ret_t }) = info.as_addr_ref() {
            sym_table.new_address(name, translate_addr(*addr), ret_t);
        }
    }

    Info {
        number_of_imported_strings,
        number_of_imported_instructions,
        number_of_imported_global_methods: 0,
    }
}
