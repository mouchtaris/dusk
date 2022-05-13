use super::{te, Result};

pub fn parse_invocation(src: &str) -> Result<ast::Invocation> {
    let lex = parse::lex::Lex::new(src);
    let parser = parse::dust::InvocationParser::new();
    let res = parser.parse(lex).map_err(parse::map_err(src));
    Ok(te!(res))
}

pub fn parse_expr(src: &str) -> Result<ast::Expr> {
    let lex = parse::lex::Lex::new(src);
    let parser = parse::dust::ExprParser::new();
    let res = parser.parse(lex).map_err(parse::map_err(src));
    Ok(te!(res))
}

pub fn parse_block(src: &str) -> Result<ast::Block> {
    let lex = parse::lex::Lex::new(src);
    let parser = parse::dust::BlockParser::new();
    let res = parser.parse(lex).map_err(parse::map_err(src));
    Ok(te!(res))
}
