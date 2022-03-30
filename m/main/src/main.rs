use {::error::te, std::fs};

use main::Result;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let sample_path = args.get(1).map(|s| s.as_str()).unwrap_or("sample.dust");
    log::debug!("Loading {}", sample_path);
    let sample_text: String = te!(fs::read_to_string(sample_path));

    log::debug!("Parsing {}", sample_path);
    let module_ast = te!(parse::parse(&sample_text));
    log::trace!("AST: {:#?}", module_ast);

    let mut cmp = compile::Compiler::new();
    te!(cmp.init());
    cmp = te!(cmp.compile(module_ast));
    use show::Show;
    te!(cmp.write_to(fs::File::create("_.compiler.txt")));

    let icode = if cfg!(release) {
        cmp.icode
    } else {
        // Try out ser-deser, to catch breaks
        te!(main::sd::copy(&cmp)).icode
    };

    let mut vm = vm::Vm::default();
    vm.reset();
    vm.init();
    te!(vm.init_bin_path_from_path_env());
    te!(vm.eval_icode(&icode));
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    Ok(())
}
