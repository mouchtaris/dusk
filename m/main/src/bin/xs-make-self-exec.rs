use {
    error::te,
    std::{env, fs, io},
};

error::Error! {
    Io = io::Error
}

fn main() -> Result<()> {
    let inp_path: String = te!(env::args().skip(1).next(), "missing input_path");

    let inp: Vec<u8> = te!(std::fs::read(&inp_path), "read {}", inp_path);
    let mut file: fs::File = te!(fs::File::create(&inp_path), "write {}", inp_path);

    use io::Write;
    te!(file.write_all("#!/bin/env xs-run\n".as_bytes()));
    te!(file.write_all(&inp));

    Ok(())
}
