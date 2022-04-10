use {
    super::{fs, io, sd, Result},
    error::{ldebug, te, temg},
};

pub fn load_icode(input_path: &str) -> Result<vm::ICode> {
    Ok(te!(load_compiler(input_path)).icode)
}

pub fn load_compiler(input_path: &str) -> Result<compile::Compiler> {
    ldebug!("Loading {}", input_path);
    Ok(te!(read_compiler(&mut te!(fs::File::open(input_path)))))
}

pub fn read_compiler<R: io::Read>(mut input: R) -> Result<compile::Compiler> {
    let inp: Vec<u8> = {
        let mut inp: Vec<u8> = vec![];
        te!(io::Read::read_to_end(&mut input, &mut inp));

        if !inp.is_empty() && inp[0] == b'#' {
            let len = inp.len();
            let hashbang_end = inp.iter().cloned().take_while(|&b| b != b'\n').count();
            let hashbang_seg = te!(std::str::from_utf8(&inp[0..=hashbang_end]));
            ldebug!(
                "truncate {} {}:{:?} -> {:x}",
                len,
                hashbang_seg.len(),
                hashbang_seg,
                inp[hashbang_end + 1]
            );
            inp.copy_within(hashbang_end + 1.., 0);
            inp.truncate(len - hashbang_end - 1);
        }

        inp
    };

    ldebug!("loading {} -> {:x}", inp.len(), &inp[0]);
    let cmp = te!(sd::deser(inp.as_slice()));
    Ok(cmp)
}

pub fn make_vm() -> Result<vm::Vm> {
    let mut vm = vm::Vm::default();
    vm.reset();
    te!(vm.init_bin_path_from_path_env());
    Ok(vm)
}

pub fn make_vm_call(
    vm: &mut vm::Vm,
    cmp: &compile::Compiler,
    func_addr: &str,
    args: Vec<String>,
) -> Result<()> {
    let sinfo = compile::scopes(&cmp.sym_table).find(|&(_, name, _)| name == func_addr);
    Ok(match sinfo {
        None => temg!("Function not found: {}", func_addr),
        Some((_, _, sinfo)) => {
            let addr = te!(sinfo.as_addr_ref()).addr;
            vm.init(args);
            vm.jump(addr);
            te!(vm.eval_icode(&cmp.icode));
            te!(vm::Instr::CleanUp(0).operate_on(vm));
        }
    })
}

pub fn list_func(cmp: &compile::Compiler) -> impl Iterator<Item = &str> {
    use collection::Recollect;
    compile::scopes(cmp)
        .filter_map(|l| match l {
            (1, name, info) if !name.starts_with('_') && info.as_addr_ref().is_ok() => Some(name),
            _ => None,
        })
        .sorted()
        .into_iter()
}
