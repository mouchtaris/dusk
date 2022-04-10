use {
    ::error::te,
    main::Result,
    std::{fs, io},
};

fn main() -> Result<()> {
    te!(main::init());

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    let input: Box<dyn io::Read> = match args.pop().as_ref().map(String::as_str) {
        Some("-") | None => Box::new(io::stdin()),
        Some(path) => te!(fs::File::open(path).map(Box::new)),
    };
    let output: Box<dyn io::Write> = match args.pop().as_ref().map(String::as_str) {
        Some("-") | None => Box::new(io::stdout()),
        Some(path) => te!(fs::File::open(path).map(Box::new)),
    };

    let icode = te!(main::read_compiler(input));

    use show::Show;
    te!(icode.write_to(Ok(output)));

    Ok(())
}
