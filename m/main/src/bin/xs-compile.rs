use {
    ::error::te,
    main::{errors, sd, Result},
    std::{fs, io},
};

fn main() {
    errors::main(main_app);
}

fn main_app() -> Result<()> {
    te!(main::init());

    let args = std::env::args().collect::<Vec<_>>();

    let input_path = args.get(1).map(String::as_str).unwrap_or("-");
    let output_path = args.get(2).map(String::as_str).unwrap_or("-");

    log::info!("Loading {}", input_path);
    let input_path = match input_path {
        "-" => "/dev/stdin",
        x => x,
    };
    let input_text: String = te!(
        fs::read_to_string(input_path),
        "Reading input file: {}",
        input_path
    );

    log::info!("Parsing {}", input_path);
    let module_ast = te!(parse::parse(&input_text));
    #[cfg(not(feature = "release"))]
    {
        use io::Write;
        te!(te!(fs::File::create("_.ast.txt")).write_fmt(format_args!("{:#?}", module_ast)));
    }

    log::info!("Compiling {}", input_path);
    let mut cmp = compile::Compiler::new();
    te!(cmp.init(&input_path));
    te!(cmp.compile(module_ast));
    #[cfg(feature = "debug")]
    {
        use show::Show;
        te!(cmp.write_to(fs::File::create("_.compiler.txt")));
    }

    let output_path = match output_path {
        "-" => "/dev/stdout",
        x => x,
    };
    log::info!("Writing to {}", output_path);
    let dst = te!(fs::File::create(&output_path), "Writing to {}", output_path);
    te!(sd::ser(dst, &cmp), "Serializing to {}", output_path);

    Ok(())
}
