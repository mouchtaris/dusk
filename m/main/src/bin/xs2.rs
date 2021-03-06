use {::error::te, std::fs};

use main::Result;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    args.reverse();

    let sample_path = args.pop().unwrap_or("/dev/stdin".to_owned());
    log::debug!("Loading {}", sample_path);
    let sample_text: String = te!(fs::read_to_string(&sample_path));

    log::debug!("Parsing {}", sample_path);
    let module_ast = te!(parse::parse(&sample_text));
    log::trace!("AST: {:#?}", module_ast);

    let mut cmp = compile::Compiler::new();
    te!(cmp.init(&sample_path));
    te!(cmp.compile(module_ast));

    #[cfg(not(feature = "release"))]
    {
        use show::Show;
        te!(cmp.write_to(fs::File::create("_.compiler.txt")));
    }

    let icode = if cfg!(release) {
        cmp.icode
    } else {
        // Try out ser-deser, to catch breaks
        te!(main::sd::copy(&cmp)).icode
    };

    let mut vm = te!(main::make_vm());
    te!(vm.init(args));
    te!(vm.eval_icode(&icode));
    #[cfg(not(feature = "release"))]
    te!(vm.write_to(fs::File::create("./_.vm.txt")));

    Ok(())
}
