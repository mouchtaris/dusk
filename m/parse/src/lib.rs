pub const VERSION: &str = "0.0.1";

pub use ::lex;

use {lalrpop_util::lalrpop_mod, std::fmt};

lalrpop_mod!(pub dust);

error::Error![Lalrpop = String];

pub fn parse(s: &str) -> Result<ast::Module> {
    let inp = lex::Lex::new(s);
    dust::ModuleParser::new().parse(inp).map_err(map_err)
}

pub fn map_err<L, T, E>(e: lalrpop_util::ParseError<L, T, E>) -> Error
where
    L: fmt::Debug,
    T: fmt::Debug,
    E: fmt::Debug,
{
    error::IntoResult::<_, ()>::into_result(Err(format!("{:?}", e)))
        .err()
        .unwrap()
}
