use {
    ::error::te,
    main::sd,
    std::{fs, io},
};

error::Error! {
    Vm = vm::Error
    Io = io::Error
    Main = main::Error
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = te!(args.get(1));

    log::debug!("Loading {}", input_path);
    let inp: Vec<u8> = {
        let mut inp: Vec<u8> = te!(fs::read(input_path));
        if inp[0] == b'#' {
            let len = inp.len();
            let hashbang_end = inp.iter().cloned().take_while(|&b| b != b'\n').count();
            inp.copy_within(hashbang_end + 1.., 0);
            inp.truncate(len - hashbang_end);
        }
        inp
    };
    let icode: vm::ICode = {
        let cmp: compile::Compiler = te!(sd::deser(inp.as_slice()));
        cmp.icode
    };

    let mut vm = vm::Vm::default();
    vm.reset();
    te!(vm.init_bin_path_from_path_env());
    vm = te!(vm.load_icode(&icode));

    #[cfg(not(release))]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    let _ = vm;

    Ok(())
}
