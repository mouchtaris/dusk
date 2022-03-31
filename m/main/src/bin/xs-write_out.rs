use {
    error::te,
    main::Result,
    std::{env, fs, io},
};
/// Write the first argument as content to every other argument as a
/// file path.
///
fn main() -> Result<()> {
    let mut argv = env::args().into_iter();
    let name = te!(argv.next());

    match name.as_str() {
        "xs-write_in" | "write_in" => {
            let mut files = vec![];
            for path in argv {
                let file = te!(fs::File::create(&path));
                let file = io::BufWriter::new(file);
                files.push(file)
            }
            use bio::{Bytes1K, Buffer};
            let mut buf = Buffer::new(Bytes1K::new());

            let mut stdin = io::stdin();
            while te!(buf.read_from(&mut stdin)) > 0 {
                for file in &mut files {
                    te!(buf.write_to(file));
                    buf.compact();
                }
            }
        }
        other => panic!("{:?}", other),
    }
    Ok(())
}
