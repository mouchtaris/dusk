use {::error::te, main::Result, std::fs};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    args.reverse();

    let input_path = te!(args.pop(), "Missing input path");
    let icode = te!(main::load_icode(&input_path));

    let mut vm = te!(main::make_vm(args));
    te!(vm.debug_icode(&icode));

    #[cfg(not(release))]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    let _ = vm;

    Ok(())
}
