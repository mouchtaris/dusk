use {
    ::error::te,
    main::sd,
    std::{fs, io},
};

error::Error! {
    Io = io::Error
    Parse = parse::Error
    Compile = compile::Error
    Main = main::Error
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = te!(args.get(1));
    let output_path = te!(args.get(2));

    log::info!("Loading {}", input_path);
    let input_text: String = te!(fs::read_to_string(input_path));

    log::info!("Parsing {}", input_path);
    let module_ast = te!(parse::parse(&input_text));
    #[cfg(not(release))]
    {
        use io::Write;
        te!(te!(fs::File::create("_.ast.txt")).write_fmt(format_args!("{:#?}", module_ast)));
    }

    log::info!("Compiling {}", input_path);
    let mut cmp = compile::Compiler::new();
    te!(cmp.init());
    cmp = te!(cmp.compile(module_ast));
    #[cfg(not(release))]
    {
        use show::Show;
        te!(cmp.write_to(fs::File::create("_.compiler.txt")));
    }

    let dst = te!(fs::File::create(&output_path), "{}", output_path);
    te!(sd::ser(dst, &cmp));

    Ok(())
}
