use {::error::te, main::Result};

fn main() {
    main::run_main(xs_call);
}

fn xs_call() -> Result<()> {
    te!(main::init());

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let mut args = te!(if_source_newer_than_target(args));

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
    te!(main::make_vm_call(&mut vm, &compl, &func_addr, args)
        .map_err(|err| err.with_comment(format!("Loading icode from {}", module_path))));
    Ok(())
}

fn last_mod(path: impl AsRef<std::path::Path>) -> Result<std::time::SystemTime> {
    Ok(te!(te!(std::fs::metadata(path)).modified()))
}

fn if_source_newer_than_target(mut args: Vec<String>) -> Result<Vec<String>> {
    let iter = args.iter().enumerate();

    const PATS: &[&str] = &["--if-source-newer=", "--than-target="];

    let now = std::time::SystemTime::now();
    let mut mods = [now, now];
    let mut idxs = [0, 0];

    for (i, arg) in iter {
        for (j, pat) in PATS.iter().copied().enumerate() {
            if let Some(path) = arg.strip_prefix(pat) {
                mods[j] = te!(last_mod(path));
                idxs[j] = i;
            }
        }
    }

    let [source_mod, target_mod] = mods;

    match () {
        _ if target_mod == source_mod => {}
        _ if target_mod > source_mod => {
            std::process::exit(0);
        }
        _ => {
            idxs.sort();
            let [j, i] = idxs;
            args.swap_remove(i);
            args.swap_remove(j);
        }
    }

    Ok(args)
}
