use {
    ::error::te,
    std::{fs, io},
};

error::Error! {
    Msg = String
    Io = io::Error
    Parse = parse::Error
    Vm = vm::Error
    Compile = compile::Error
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let sample_path = args.get(1).map(|s| s.as_str()).unwrap_or("sample.dust");
    log::debug!("Loading {}", sample_path);
    let sample_text: String = te!(fs::read_to_string(sample_path));

    log::debug!("Parsing {}", sample_path);
    let module_ast = te!(parse::parse(&sample_text));
    log::trace!("AST: {:#?}", module_ast);

    let mut cmp = compile::Compiler::default();
    cmp.init();
    cmp = te!(cmp.compile(&module_ast));
    te!(cmp.write_to(fs::File::create("_.compiler.txt")));

    let mut vm = vm::Vm::default();
    vm.reset();
    // te!(vm.init_bin_path_from_path_env());
    vm.init_bin_path_system();
    vm = te!(vm.load_icode(&cmp.icode));
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    Ok(())
}
