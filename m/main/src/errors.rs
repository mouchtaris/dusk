use {
    super::{cli, init, te, Error, ErrorKind, Result},
    std::{fmt, process},
};

pub fn main_app(main_app: impl cli::Cmd) {
    main(|| {
        let args = std::env::args().collect::<Vec<_>>();
        te!(init());
        te!(main_app(args));
        Ok(())
    })
}

pub fn main<M>(main_app: M)
where
    M: FnOnce() -> Result<()>,
{
    match main_app() {
        Ok(_) => return,
        Err(err) => show_error(err),
    }

    process::exit(1);
}

fn show_trace(trace: error::Trace) {
    eprintln!("Source trace ({}):", trace.len());
    for (f, l, cs) in trace {
        let _ = (f, l);
        for c in cs {
            eprintln!(" - {}", c);
        }
    }
}

fn show_message<S: fmt::Display>(trace: error::Trace, msg: S) {
    show_trace(trace);
    eprintln!("{}", msg);
}

fn show_error(err: Error) {
    let Error { kind, trace } = err;
    match kind {
        ErrorKind::Msg(msg) => show_message(trace, msg),
        ErrorKind::Compile(err) => show_compile_error(err),
        ErrorKind::Parse(err) => show_parse_error(err),
        ErrorKind::Io(io) => show_message(trace, format_args!("{io:?}")),
        other => panic!("{:?}", other),
    }
}

fn show_compile_error(err: compile::Error) {
    use compile::ErrorKind;
    let compile::Error { kind, trace } = err;
    match kind {
        ErrorKind::ParseDust(err) => show_parse_error(err),
        ErrorKind::Message(msg) => show_message(trace, msg),
        ErrorKind::Io(io) => show_message(trace, format_args!("{:?}", io)),
        other => panic!("{:?}", other),
    }
}

fn show_parse_error(err: parse::Error) {
    use parse::ErrorKind;
    let parse::Error { kind, .. } = err;
    match kind {
        ErrorKind::Lalrpop(err) => show_lalrpop_error(err),
        other => panic!("{:?}", other),
    }
}

fn show_lalrpop_error((inp, err): parse::SourceError) {
    use parse::LocationError as P;
    const NO_EXPECT: &[String] = &[];
    let show = |a, b, e| show_source(&inp, a, b, e);
    match err {
        P::InvalidToken { location } => {
            show(location, None, NO_EXPECT);
            show_error_text("Invalid token");
        }
        P::UnrecognizedEof { location, expected } => {
            show(location, None, expected.as_slice());
            show_error_text("Premature EOF");
        }
        P::UnrecognizedToken {
            token: (a, _, b),
            expected,
        } => {
            show(a, Some(b), expected.as_slice());
            show_error_text("Unrecognized token");
        }
        P::ExtraToken { token: (a, _, b) } => {
            show(a, Some(b), NO_EXPECT);
            show_error_text("Extra token");
        }
        P::User { .. } => panic!("Impossible"),
    }
}

fn show_source<ExpectedIter>(source: &str, start: usize, end: Option<usize>, expected: ExpectedIter)
where
    ExpectedIter: IntoIterator,
    ExpectedIter::Item: fmt::Display,
{
    struct Cyclon<T, const N: usize> {
        data: [T; N],
        idx: usize,
    }
    impl<T: Copy, const N: usize> Cyclon<T, N> {
        fn new(t: T) -> Self {
            Self {
                data: [t; N],
                idx: 0,
            }
        }
        fn incr(idx: &mut usize) {
            *idx = (*idx + 1) % N;
        }
        fn push(&mut self, t: T) {
            let Self { data, idx } = self;
            data[*idx] = t;
            Self::incr(idx);
        }
        fn reset(&mut self) {
            self.idx = 0
        }
        fn is_full(&self) -> bool {
            self.idx == N - 1
        }
        fn iter(&self) -> impl Iterator<Item = T> {
            let &Self { data, mut idx } = self;
            let mut i = 0;
            std::iter::from_fn(move || {
                if i == N {
                    None
                } else {
                    let t = data[idx];
                    Self::incr(&mut idx);
                    i += 1;
                    Some(t)
                }
            })
        }
        fn iter2(&self) -> impl Iterator<Item = T> {
            let &Self { data, idx } = self;
            let mut i = 0;
            std::iter::from_fn(move || {
                if i == idx {
                    None
                } else {
                    let t = data[i];
                    i += 1;
                    Some(t)
                }
            })
        }
    }

    type Cyc<'s> = Cyclon<&'s str, CTX_LEN>;
    const CTX_LEN: usize = 5;
    let mut ctx = Cyc::new("");
    let mut chars = source.chars();
    let mut line_start = 0;
    let mut offset = 0;
    let mut line_count = 0;

    let mut push_line = |line_end| {
        let line = &source[line_start..line_end];
        ctx.push(line);
        line_start = line_end;
    };

    let err_char: char = loop {
        if let Some(chr) = chars.next() {
            let bytelen = chr.len_utf8();
            offset += bytelen;

            if offset - bytelen == start {
                push_line(offset - bytelen);
                break chr;
            }

            if chr == '\n' {
                push_line(offset);
                line_count += 1;
            }
        } else {
            break '^';
        }
    };
    ctx.iter().enumerate().for_each(|(i, line)| {
        color(
            249 + i as u8,
            format_args!(" {:6} |  {}", line_count - CTX_LEN + i, line),
        )
    });

    color(142, err_char);

    ctx.reset();
    line_start = offset;
    while let Some(chr) = chars.next() {
        let bytelen = chr.len_utf8();
        offset += bytelen;

        if chr == '\n' {
            let line = &source[line_start..offset];
            line_start = offset;
            let full = ctx.is_full();
            ctx.push(line);
            if full {
                break;
            }
        }
    }
    let mut after = ctx.iter2();
    if let Some(line) = after.next() {
        color((249 + CTX_LEN) as u8, line)
    }
    after.enumerate().for_each(|(i, line)| {
        color(
            (249 + CTX_LEN - i - 1) as u8,
            format_args!(" {:6} |  {}", line_count - CTX_LEN + i, line),
        )
    });
}

fn show_error_text<M: fmt::Display>(msg: M) {
    color(135, msg);
}

fn color<M: fmt::Display>(col: u8, msg: M) {
    eprint!("\x1b[38;5;{col}m{msg}\x1b[m", col = col, msg = msg);
}
