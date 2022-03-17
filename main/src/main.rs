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

    const SAMPLE_PATH: &str = "test.dust";
    log::debug!("Loading {}", SAMPLE_PATH);
    let sample_text: String = te!(fs::read_to_string(SAMPLE_PATH));

    log::debug!("Parsing {}", SAMPLE_PATH);
    let module_ast = te!(parse::parse(&sample_text));
    log::trace!("AST: {:#?}", module_ast);

    let mut cmp = compile::Compiler::default();
    cmp.init();
    te!(cmp.compile(&module_ast));
    log::debug!("Compiler: {:#?}", cmp);

    let mut vm = vm::Vm::default();
    vm.reset();
    te!(vm.init_bin_path_from_path_env());
    vm = te!(vm.load_icode(&cmp.icode));
    log::debug!("VM: {:#?}", vm);

    Ok(())
}
