use {::error::te, main::Result};

fn main() -> Result<()> {
    te!(main::init());

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    args.reverse();

    let mut module_path = te!(args.pop(), "Missing module path");
    let func_addr = te!(args.pop(), "Missing function addr");

    if module_path == "-" {
        module_path.clear();
        module_path.push_str("/dev/stdin");
    }

    let compl = te!(
        main::load_compiler(&module_path),
        "ICode loading {}",
        module_path
    );

    let mut vm = te!(main::make_vm());
    te!(main::make_vm_call(&mut vm, &compl, &func_addr, args));
    Ok(())
}
