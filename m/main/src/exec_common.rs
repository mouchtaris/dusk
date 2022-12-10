use super::{env, fmt, fs, io, te, Error, Result};

pub fn run_main(main: impl FnOnce() -> Result<()>) {
    match main() {
        Ok(r) => r,
        Err(err) => handle_error(err),
    }
}

fn handle_error(err: Error) {
    write_err(error_socket_from_env().unwrap_or_else(|| Box::new(io::stderr())))(err)
}

fn write_err(mut dest: impl io::Write) -> impl FnOnce(Error) {
    move |err| {
        let mut f = || -> Result<()> {
            te!(writeln!(dest, "{:?}", err));
            Ok(())
        };
        match f() {
            Ok(r) => r,
            Err(err2) => {
                write_stderr(err2.with_comment("Writing to err dest"));
                write_stderr(err);
            }
        }
    }
}

fn write_stderr(err: Error) {
    write_err(io::stderr())(err);
}

fn error_socket_from_env() -> Option<Box<dyn io::Write>> {
    let f = || -> Result<Option<Box<dyn io::Write>>> {
        if let Ok(path) = env::var("DUSTERR") {
            let dest = te!(
                fs::File::options().append(true).open(&path),
                "Opening DUSTERR path: {path}"
            );
            Ok(Some(Box::new(dest)))
        } else {
            Ok(None)
        }
    };

    match f() {
        Ok(r) => r,
        Err(err) => {
            write_stderr(err);
            None
        }
    }
}
