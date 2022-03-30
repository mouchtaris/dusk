use {::error::te, main::Result, std::fs};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = te!(args.get(1));
    let icode = te!(main::load_icode(&input_path));

    let mut vm = vm::Vm::default();
    vm.reset();
    vm.init();
    te!(vm.init_bin_path_from_path_env());
    te!(vm.debug_icode(&icode));

    #[cfg(not(release))]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    let _ = vm;

    Ok(())
}
