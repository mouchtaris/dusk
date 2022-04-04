use {error::te, main::Result, std::env};

fn main() -> Result<()> {
    let mut args: Vec<_> = env::args().collect();
    args.reverse();
    args.pop();

    let path = te!(args.pop(), "Missing input path");
    let cmp = te!(main::load_compiler(&path));
    for f in main::list_func(&cmp) {
        print!("{}\x00", f);
    }
    Ok(())
}
