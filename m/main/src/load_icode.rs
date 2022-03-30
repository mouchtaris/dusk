use {
    super::{fs, sd, Result},
    error::{ldebug, te},
};

pub fn load_icode(input_path: &str) -> Result<vm::ICode> {
    ldebug!("Loading {}", input_path);
    let inp: Vec<u8> = {
        let mut inp: Vec<u8> = te!(fs::read(input_path));
        if inp[0] == b'#' {
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
    let icode: vm::ICode = {
        ldebug!("loading {} -> {:x}", inp.len(), &inp[0]);
        let cmp: compile::Compiler = te!(sd::deser(inp.as_slice()));
        cmp.icode
    };
    Ok(icode)
}

pub fn make_vm(args: Vec<String>) -> Result<vm::Vm> {
    let mut vm = vm::Vm::default();
    vm.reset();
    vm.init(args);
    te!(vm.init_bin_path_from_path_env());
    Ok(vm)
}
