use {
    ::error::{ldebug, te},
    main::sd,
    std::{fs, io},
};

error::Error! {
    Vm = vm::Error
    Io = io::Error
    Main = main::Error
    Utf8 = std::str::Utf8Error
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = te!(args.get(1));

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

    let mut vm = vm::Vm::default();
    vm.reset();
    vm.init();
    te!(vm.init_bin_path_from_path_env());
    vm = te!(vm.load_icode(&icode));

    #[cfg(not(release))]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    let _ = vm;

    Ok(())
}
