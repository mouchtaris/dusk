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
    let inp = te!(fs::File::open(input_path), "{}", input_path);
    //let icode = te!(vm::ICode::load_from());
    let icode = {
        let cmp: compile::Compiler = te!(sd::deser(inp));
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
