use {
    ::error::te,
    std::{fs, io},
};

error::Error! {
    Io = io::Error
    Parse = parse::Error
    Compile = compile::Error
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = te!(args.get(1));
    let output_path = te!(args.get(2));

    log::debug!("Loading {}", input_path);
    let input_text: String = te!(fs::read_to_string(input_path));

    log::debug!("Parsing {}", input_path);
    let module_ast = te!(parse::parse(&input_text));
    log::trace!("AST: {:#?}", module_ast);

    log::debug!("Compiling {}", input_path);
    let mut cmp = compile::Compiler::default();
    cmp.init();
    cmp = te!(cmp.compile(&module_ast));
    #[cfg(debug)]
    {
        use show::Show;
        te!(cmp.write_to(fs::File::create("_.compiler.txt")));
    }

    te!(cmp.icode.write_to(fs::File::create(&output_path)));

    Ok(())
}
