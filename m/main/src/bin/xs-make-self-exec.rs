use {
    error::te,
    main::Result,
    std::{env, io},
};

fn main() -> Result<()> {
    let argv = env::args().skip(1).collect::<Vec<_>>();

    // TODO switch order
    let mut input = te!(main::args_get_input(argv.iter()));
    let header: &String = te!(argv.get(1), "missing header");

    let mut inp = <_>::default();
    te!(io::Read::read_to_end(&mut input, &mut inp));

    // !! Important: AFTER INPUT or the input file might become empty.
    let mut output = te!(main::args_get_output(argv.iter()));

    use io::Write;
    te!(write!(output, "{}\n", header));
    te!(output.write_all(&inp));

    Ok(())
}
