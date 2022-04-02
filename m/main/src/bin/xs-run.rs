use {::error::te, main::Result, std::fs};

fn main() -> Result<()> {
    te!(main::init());

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    args.reverse();

    let input_path = te!(args.pop(), "Missing input path");
    let icode = te!(main::load_icode(&input_path));

    let mut vm = te!(main::make_vm());
    vm.init(args);
    te!(vm.eval_icode(&icode));

    #[cfg(not(feature = "release"))]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    let _ = vm;

    Ok(())
}
