pub const VERSION: &str = "0.0.1";

pub use ::lex;

use {error::te, lalrpop_util::lalrpop_mod};

lalrpop_mod!(dust);

error::Error![Lalrpop = String];

pub fn parse(s: &str) -> Result<ast::Module> {
    let inp = lex::Lex::new(s);
    Ok(te!(dust::ModuleParser::new()
        .parse(inp)
        .map_err(|e| format!("{:?}", e))))
}
