use {::error::te, std::fs};

error::Error! {
    Vm = vm::Error
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = te!(args.get(1));

    log::debug!("Loading {}", input_path);
    let icode = te!(vm::ICode::load_from(fs::File::open(input_path)));

    let mut vm = vm::Vm::default();
    vm.reset();
    te!(vm.init_bin_path_from_path_env());
    vm = te!(vm.load_icode(&icode));

    #[cfg(debug)]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    let _ = vm;

    Ok(())
}
