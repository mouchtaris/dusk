use {
    error::{te, temg},
    io::Write,
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
        n if n.ends_with("write_in") => xs_write_in(argv),
        n if n.ends_with("write_out") => xs_write_out(argv),
        other => panic!("{:?}", other),
    }
}

fn xs_write_out<Args>(mut argv: Args) -> Result<()>
where
    Args: Iterator,
    Args::Item: AsRef<str>,
{
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    while let Some(arg) = argv.next() {
        let __ = match arg.as_ref() {
            "--stdin" =>
                io::copy(&mut stdin, &mut stdout)
                .map(|_| ()),
            a if a.starts_with("--echo=") => stdout.write_all(&a["--echo=".len()..].as_bytes()),
            a if a.starts_with("--echo") => {
                let cont = te!(argv.next(), "Missing --echo arg");
                let cont = cont.as_ref();
                stdout.write_all(cont.as_bytes())
            }
            other => temg!("Unbearable argument: {:?}", other),
        };
        te!(__);
    }
    Ok(())
}

fn xs_write_in<Args>(argv: Args) -> Result<()>
where
    Args: Iterator,
    Args::Item: AsRef<std::path::Path>,
{
    let mut files = vec![];
    for path in argv {
        let file = te!(fs::File::create(&path));
        let file = io::BufWriter::new(file);
        files.push(file)
    }

    let mut stdin = io::stdin();
    let mut buf = vec![0u8; 256];

    loop {
        use io::{Read, Write};

        let n = te!(stdin.read(&mut buf));

        if n == 0 {
            break;
        }

        let view = &buf[0..n];

        for file in &mut files {
            te!(file.write_all(view));
        }
    }

    Ok(())
}