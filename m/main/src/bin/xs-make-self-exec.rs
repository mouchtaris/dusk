use {
    error::te,
    std::{env, fs, io},
};

error::Error! {
    Io = io::Error
}

fn main() -> Result<()> {
    let mut argv = env::args().skip(1);
    let inp_path: String = te!(argv.next(), "missing input_path");
    let header: String = te!(argv.next(), "missing header");

    let inp: Vec<u8> = te!(std::fs::read(&inp_path), "read {}", inp_path);
    let mut file: fs::File = te!(fs::File::create(&inp_path), "write {}", inp_path);

    use io::Write;
    te!(write!(file, "{}\n", header));
    te!(file.write_all(&inp));

    Ok(())
}
