use {
    super::{Error, ErrorKind},
    std::fmt,
};

pub fn show_trace(mut w: impl W, trace: error::Trace) -> R {
    writeln!(w, "Source trace ({}):", trace.len())?;
    for (f, l, cs) in trace {
        let _ = (f, l);
        writeln!(w, " - {f}:{l}")?;
        for c in cs {
            writeln!(w, "   -> {c}")?;
        }
    }
    Ok(())
}

fn show_message<S: fmt::Display>(mut w: impl W, trace: error::Trace, msg: S) -> R {
    writeln!(w, "{}", msg)?;
    show_trace(w, trace)?;
    Ok(())
}

pub fn show_error(w: impl W, err: Error) -> R {
    let Error { kind, trace } = err;
    let to_err = |kind: ErrorKind| -> Error {
        Error {
            kind,
            trace: <_>::default(),
        }
    };
    match kind {
        ErrorKind::Msg(msg) => show_message(w, trace, msg),
        ErrorKind::Compile(err) => show_compile_error(w, err),
        ErrorKind::Parse(err) => show_parse_error(w, err),
        ErrorKind::Io(io) => show_message(w, trace, format_args!("{io:?}")),
        ErrorKind::Vm(err) => show_message(w, trace, format_args!("{err:?}")),
        kind @ ErrorKind::None(()) => show_message(w, trace, format_args!("{:?}", to_err(kind))),
        //ErrorKind::None(()) => show_message(trace, format_args!("None Option")),
        other => panic!("{:?}", other),
    }
}

fn show_compile_error(w: impl W, err: compile::Error) -> R {
    use compile::ErrorKind;
    let compile::Error { kind, trace } = err;
    match kind {
        ErrorKind::ParseDust(err) => show_parse_error(w, err),
        ErrorKind::Message(msg) => show_message(w, trace, msg),
        ErrorKind::Io(io) => show_message(w, trace, format_args!("{:?}", io)),
        other => panic!("{:?}", other),
    }
}

fn show_parse_error(w: impl W, err: parse::Error) -> R {
    use parse::ErrorKind;
    let parse::Error { kind, .. } = err;
    match kind {
        ErrorKind::Lalrpop(err) => show_lalrpop_error(w, err),
        other => panic!("{:?}", other),
    }
}

fn show_lalrpop_error(mut w: impl io::Write, (inp, err): parse::SourceError) -> R {
    use parse::LocationError as P;
    const NO_EXPECT: &[String] = &[];
    let mut show = |a, b, e| show_source(&mut w, &inp, a, b, e);
    match err {
        P::InvalidToken { location } => {
            show(location, None, NO_EXPECT)?;
            show_error_text(w, "Invalid token")
        }
        P::UnrecognizedEof { location, expected } => {
            show(location, None, expected.as_slice())?;
            show_error_text(w, "Premature EOF")
        }
        P::UnrecognizedToken {
            token: (a, _, b),
            expected,
        } => {
            show(a, Some(b), expected.as_slice())?;
            show_error_text(w, "Unrecognized token")
        }
        P::ExtraToken { token: (a, _, b) } => {
            show(a, Some(b), NO_EXPECT)?;
            show_error_text(w, "Extra token")
        }
        P::User { .. } => panic!("Impossible"),
    }
}

fn show_source<ExpectedIter>(
    mut w: impl io::Write,
    source: &str,
    start: usize,
    _end: Option<usize>,
    _expected: ExpectedIter,
) -> R
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
    ctx.iter().enumerate().try_for_each(|(i, line)| {
        color(
            &mut w,
            249 + i as u8,
            format_args!(
                " {:6} |  {}",
                if line_count >= CTX_LEN - 1 {
                    line_count - CTX_LEN + i
                } else {
                    0
                },
                line
            ),
        )
    })?;

    color(&mut w, 142, err_char)?;

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
        color(&mut w, (249 + CTX_LEN) as u8, line)?
    }
    after.enumerate().try_for_each(|(i, line)| {
        color(
            &mut w,
            (249 + CTX_LEN - i - 1) as u8,
            format_args!(" {:6} |  {}", line_count - CTX_LEN + i, line),
        )
    })
}

fn show_error_text<M: fmt::Display>(w: impl W, msg: M) -> R {
    color(w, 135, msg)
}

fn color<M: fmt::Display>(mut w: impl W, col: u8, msg: M) -> R {
    write!(w, "\x1b[38;5;{col}m{msg}\x1b[m", col = col, msg = msg)
}
type R = std::io::Result<()>;
use io::Write as W;
use std::io;
