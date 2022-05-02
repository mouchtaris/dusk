use {::error::te, main::Result};

fn main() -> Result<()> {
    te!(main::init());

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    args.reverse();

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

    let icode = te!(main::load_icode(&input_path));

    let mut vm = te!(main::make_vm());
    te!(vm.init(args));
    te!(vm.debug_icode(&icode));
    Ok(())
}
