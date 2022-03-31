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
    let mut buf = buffer_new();

    while let Some(arg) = argv.next() {
        let __ = match arg.as_ref() {
            "--stdin" => io_copy(&mut stdin, &mut stdout, &mut buf),
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
    let mut buf = buffer_new();

    let mut stdin = io::stdin();
    while te!(buf.read_from(&mut stdin)) > 0 {
        for file in &mut files {
            te!(buf.write_to(file));
            buf.compact();
        }
    }

    Ok(())
}

type Buffer = bio::Buffer<bio::Bytes1K>;

fn buffer_new() -> Buffer {
    use bio::{Buffer, Bytes1K};
    Buffer::new(Bytes1K::new())
}

fn io_copy<I, O>(mut inp: I, mut out: O, buf: &mut Buffer) -> io::Result<()>
where
    I: io::Read,
    O: io::Write,
{
    while buf.free() == 0 || buf.read_from(&mut inp)? > 0 {
        while buf.len() > 0 {
            buf.write_to(&mut out)?;
            buf.compact();
        }
    }
    Ok(())
}
