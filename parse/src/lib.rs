pub const VERSION: &str = "0.0.1";

use {error::te, lalrpop_util::lalrpop_mod};

lalrpop_mod!(dust);

error::Error![Lalrpop = String];

pub fn parse(s: &str) -> Result<ast::Module> {
    Ok(te!(dust::ModuleParser::new()
        .parse(s)
        .map_err(|e| format!("{:?}", e))))
}
