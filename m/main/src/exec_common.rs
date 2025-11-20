use super::{cli, env, errors, fs, init, io, te, Error, Result};

pub fn run_main_app(main_app: impl cli::Cmd) {
    run_main(|| {
        let args = std::env::args().collect::<Vec<_>>();
        te!(init());
        te!(main_app(args));
        Ok(())
    })
}

pub fn run_main(main: impl FnOnce() -> Result<()>) {
    match main() {
        Ok(r) => r,
        Err(err) => {
            handle_error(err);
            std::process::exit(1);
        }
    }
}

fn handle_error(err: Error) {
    write_err(error_socket_from_env().unwrap_or_else(|| Box::new(io::stderr())))(err)
}

fn write_err(dest: impl io::Write) -> impl FnOnce(Error) {
    move |err| match (|| -> Result<()> { Ok(te!(errors::show_error(dest, err))) })() {
        Ok(r) => r,
        Err(err2) => {
            write_stderr(err2.with_comment("Writing to err dest"));
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
