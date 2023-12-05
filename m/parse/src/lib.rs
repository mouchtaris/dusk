pub const VERSION: &str = "0.0.1";

pub use ::lex;
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub dust);

pub type ParseError<'s> = lalrpop_util::ParseError<usize, lex::Tok<'s>, Error>;
pub type LocationError = lalrpop_util::ParseError<usize, (), ()>;
pub type SourceError = (String, LocationError);
error::Error![Lalrpop = SourceError];

pub fn parse(source: &str) -> Result<ast::Module> {
    let inp = lex::Lex::new(source);
    dust::ModuleParser::new()
        .parse(inp)
        .map_err(map_err(source))
}

// Necessary to kill the input lifetime
pub fn map_err(inp: &str) -> impl FnOnce(ParseError) -> Error + '_ {
    |e| {
        use lalrpop_util::ParseError::*;
        type L = usize;
        type T<T> = (L, T, L);

        fn strip(tok: T<lex::Tok>) -> T<()> {
            let (a, _, b) = tok;
            (a, (), b)
        }

        let error = |e: LocationError| -> Error {
            Error {
                kind: ErrorKind::Lalrpop((inp.to_owned(), e)),
                trace: <_>::default(),
            }
        };

        match e {
            InvalidToken { location } => error(InvalidToken { location }),
            UnrecognizedEof { location, expected } => error(UnrecognizedEof { location, expected }),
            UnrecognizedToken { token, expected } => error(UnrecognizedToken {
                token: strip(token),
                expected,
            }),
            ExtraToken { token } => error(ExtraToken {
                token: strip(token),
            }),
            User { error } => error,
        }
    }
}
