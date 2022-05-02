use {::error::te, main::Result};

fn main() -> Result<()> {
    te!(main::init());

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    args.reverse();

    let input_path = args.pop();
    let input_path = match input_path {
        Some(mut s) => match s.as_str() {
            "-" => {
                s.clear();
                s.push_str("/dev/stdin");
                s
            }
            _ => s,
        },
        None => "/dev/stdin".to_owned(),
    };

    let mut bugger = te!(vm::debugger::Bugger::open());
    let mut compiler = te!(main::load_compiler(&input_path));
    // let icode = te!(main::load_icode(&input_path));
    let mut vm = te!(main::make_vm());
    let icode = compiler.icode;

    compiler.icode = <_>::default();
    bugger.callbacks.data.push(Box::new(move |vm, instr| {
        use std::io;
        let err = Ok(io::stderr());
        //te!(vm.write_to(err).map_err(Box::new));
        te!(vm_debug::write_to(vm, &compiler, err).map_err(Box::new));
        eprintln!("");
        eprintln!("");
        eprintln!("");
        eprintln!("===== ===== =====");
        eprintln!("[BUGGER] {} {:?}", vm.instr_addr(), instr);
        Ok(())
    }));

    te!(vm.init(args));
    bugger = te!(vm.debug_icode(&icode, bugger));

    te!(te!(bugger
        .receiver_thread
        .join()
        .map_err(|_| format!("Wait receiver thread"))));

    Ok(())
}
