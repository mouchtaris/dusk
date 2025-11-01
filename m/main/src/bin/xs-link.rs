use {
    ::error::te,
    compile::{sym, ScopeRef, SymbolTableExt},
    main::{sd, Compiler, Result},
    std::{collections::HashMap, io},
    vm::StringInfo,
};

fn main() {
    main::run_main(xs_link);
}

fn xs_link() -> Result<()> {
    te!(main::init());

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let modules: Result<Vec<_>> = args.iter().map(|path| main::load_compiler(&path)).collect();
    let modules = te!(modules);

    let mut cmp = Compiler::new();
    cmp.enter_scope();
    cmp.enter_scope();

    let module = modules.iter().fold(Ok(cmp), |cmp, a| -> Result<Compiler> {
        let mut cmp = te!(cmp);

        let instrs = cmp.icode.instructions.len();

        let adding_module = a;

        let strings: HashMap<&usize, &String> = adding_module
            .icode
            .strings
            .iter()
            .map(|(k, StringInfo { id })| (id, k))
            .collect();

        // Push strings, translating ids
        let mut trnstrid = |id: usize| -> usize {
            let s = strings.get(&id).unwrap();
            match cmp.icode.strings.get(*s) {
                Some(StringInfo { id }) => *id,
                None => {
                    let id = cmp.icode.strings.len();
                    assert!(cmp
                        .icode
                        .strings
                        .values()
                        .map(|StringInfo { id }| id)
                        .find(|x| **x == id)
                        .is_none());
                    cmp.icode.strings.insert((*s).to_owned(), StringInfo { id });
                    id
                }
            }
        };

        // Push instructions, translating addresses and ids
        for instr in &adding_module.icode.instructions {
            use vm::Instr::*;
            let instr = match *instr {
                Jump { addr } => Jump {
                    addr: addr + instrs,
                },
                PushStr(id) => PushStr(trnstrid(id)),
                PushFuncAddr(addr) => PushFuncAddr(addr + instrs),
                RetStr(id) => RetStr(trnstrid(id)),
                RetFuncAddr(addr) => RetFuncAddr(addr + instrs),
                _ => *instr,
            };
            cmp.icode.instructions.push_back(instr);
        }

        // Insert symbols, only if they are global and refer to func-addr.
        //
        // Translate the function address in the process.
        //
        for (name, sym_id) in adding_module
            .global_scope_opt()
            .expect("global scope")
            .symbols()
        {
            let info = sym_id.sym_info();

            if let Ok(sym::Address { addr, ret_t }) = info.as_addr_ref() {
                // Gets added to global scope anyway
                cmp.new_address(name, addr + instrs, ret_t);
            }

            //match info {
            //    sym::Info {
            //        scope_id: 1,
            //        typ: sym::Typ::Address(sym::Address { addr, ret_t }),
            //    } => {
            //        // only global scope
            //        if let Some(_) = scope.insert(
            //            name.to_owned(),
            //            sym::Info {
            //                scope_id: 1,
            //                typ: sym::Typ::Address(sym::Address {
            //                    addr: addr + instrs,
            //                    ret_t: ret_t.to_owned(),
            //                }),
            //            },
            //        ) {
            //            temg!("Double symbol: {name}")
            //        }
            //    }
            //    _ => (),
            //}
        }
        Ok(cmp)
    });
    let module = te!(module);
    let mut out = io::stdout();
    te!(sd::ser(&mut out, &module));

    Ok(())
}
