use super::{te, Result};

pub fn compile_invocation(src: &str) -> Result<ast::Invocation> {
    let lex = parse::lex::Lex::new(src);
    let parser = parse::dust::InvocationParser::new();
    let res = parser.parse(lex).map_err(parse::map_err);
    Ok(te!(res))
}
