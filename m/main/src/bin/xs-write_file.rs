use {
    error::te,
    main::Result,
    std::{env, fs},
};
/// Write the first argument as content to every other argument as a
/// file path.
///
fn main() -> Result<()> {
    let mut argv = env::args().skip(1);
    let cont = te!(argv.next(), "missing content");
    for path in argv {
        te!(fs::write(path, &cont));
    }
    Ok(())
}
