use {super::Error, std::fmt};

const _LIME: &str = "\x1b[38:5:192m";
const _GREY: &str = "\x1b[38:5:246m";
const _RED: &str = "\x1b[38:5:125m";
const _DORANGE: &str = "\x1b[38:5:220m";
const _GORANGE: &str = "\x1b[38:5:172m";
const _RESET: &str = "\x1b[m";

impl<K: fmt::Debug> fmt::Debug for Error<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (file, line, comments) in &self.trace {
            let file = *file;
            let line = *line;
            let context_message = "";
            let show = "";
            writeln!(
                f,
                "[{ERR}ERROR{RS}] {FILE}{file}{RS}:{LINE}{line}{RS} :: {explain}{show}",
                ERR = _RED,
                RS = _RESET,
                FILE = _DORANGE,
                file = file,
                LINE = _GORANGE,
                line = line,
                explain = context_message,
                show = show,
            )?;
            for comment in comments {
                writeln!(
                    f,
                    "    {GREY}[Comment]{RS} {CMNT}",
                    GREY = _GREY,
                    CMNT = comment,
                    RS = _RESET,
                )?;
            }
        }
        write!(f, "{:?}", self.kind)?;
        Ok(())
    }
}
