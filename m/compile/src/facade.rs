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

pub fn make_forward_invocation<'a>(func_name: &'a str) -> ast::Invocation<'a> {
    ast::Invocation((
        vec![],
        ast::InvocationTarget::InvocationTargetLocal(ast::InvocationTargetLocal((func_name,))),
        None,
        vec![],
        vec![],
        vec![],
        vec![ast::InvocationArg::Variable(ast::Variable(("args",)))],
    ))
}
pub fn compile_invocation_block(
    cmp: &mut super::Compiler,
    program: ast::Block,
) -> Result<super::SymInfo> {
    use super::{i, EmitExt};

    // Allocate minimal stack for call tmp local variables
    const CALL_CTX: usize = 8;
    cmp.emit1(i::Allocate { size: CALL_CTX });
    let sinfo = te!(cmp.compile(program));
    // TODO: why is this sinfo pointing to the God and not to ret-addr?
    //eprintln!("{sinfo:?}");
    //te!(super::CompileUtil::emit_cleanup(cmp, i::CleanUp, &sinfo));
    cmp.emit1(i::Return(CALL_CTX));
    return Ok(sinfo);
}
pub fn compile_invocation(cmp: &mut super::Compiler, func_name: &str) -> Result<super::SymInfo> {
    let invc = make_forward_invocation(func_name);
    let invc = ast::Expr::Invocation(invc);

    let program = ast::Block((vec![], invc));

    Ok(te!(compile_invocation_block(cmp, program)))
}
pub fn compile_wrap_in_invocation(
    func_name: &str,
    block: ast::Block,
    cmp: &mut super::Compiler,
) -> Result<super::SymInfo> {
    let program: ast::Block = te!(wrap_in_invocation(func_name, block));

    return Ok(te!(compile_invocation_block(cmp, program)));

    fn wrap_in_method<'a>(func_name: &'a str, body: ast::Block<'a>) -> ast::Item<'a> {
        let body = ast::Body::Block(body);
        let def_stmt = ast::DefStmt((func_name, body));
        ast::Item::DefStmt(def_stmt)
    }

    fn wrap_in_invocation<'a>(func_name: &'a str, body: ast::Block<'a>) -> Result<ast::Block<'a>> {
        let def_func: ast::Item = wrap_in_method(func_name, body);
        let invoc = ast::Expr::Invocation(make_forward_invocation(func_name));
        let program = ast::Block((vec![def_func], invoc));
        Ok(program)
    }
}
