use {error::te, main::Result, std::env};

fn main() -> Result<()> {
    let mut args: Vec<_> = env::args().collect();
    args.reverse();
    args.pop();

    let input_path = args.pop();
    let input_path = match input_path {
        Some(mut s) => match s.as_str() {
            "-" => {
                s.clear();
                s.push_str("/dev/stdin");
                s
            }
            _ => s,
        },
        None => "/dev/stdin".to_owned(),
    };

    let cmp = te!(main::load_compiler(&input_path));
    for f in main::list_func(&cmp) {
        print!("{}\x00", f);
    }
    Ok(())
}
