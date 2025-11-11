use {
    ::error::te,
    main::{sd, Result},
    std::io,
};

fn main() {
    main::run_main(xs_link);
}

fn xs_link() -> Result<()> {
    te!(main::init());

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let modules: Result<Vec<_>> = args.iter().map(|path| main::load_compiler(&path)).collect();
    let modules = te!(modules);

    let module = compile::link::link_modules(modules);

    let mut out = io::stdout();
    te!(sd::ser(&mut out, &module));

    Ok(())
}
