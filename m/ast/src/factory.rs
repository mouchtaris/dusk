use super::*;
pub fn invoc(name: &str) -> Invocation {
    Invocation((
        <_>::default(),
        InvocationTarget::InvocationTargetLocal(InvocationTargetLocal((&name,))),
        None,
        <_>::default(),
        <_>::default(),
        <_>::default(),
        <_>::default(),
    ))
}

pub fn let_stmt<'i, E>(name: &'i str, expr: E) -> Item<'i>
where
    E: Into<Expr<'i>>,
{
    LetStmt((name, expr.into())).into()
}

pub fn src_stmt<'i, E>(name: &'i str, expr: E) -> Item<'i>
where
    E: Into<Expr<'i>>,
{
    SrcStmt((name, expr.into())).into()
}

pub fn block_of_stmts<'i>(mut stmts: Vec<Item<'i>>, last: Item<'i>) -> Block<'i> {
    stmts.push(last);
    (stmts, EXPR_0).into()
}

pub fn expr_nat(n: &str) -> Expr {
    Expr::Natural(Natural((n,)))
}
pub fn expr_str(s: &str) -> Expr {
    Expr::String(String((s,)))
}

pub fn access_range<'i>((a, b): Tupl2<Option<Expr<'i>>>) -> Range<'i> {
    let z = || expr_nat("0");
    let mz = || expr_str("-0");
    (a.unwrap_or_else(z), b.unwrap_or_else(mz))
}

pub const EXPR_0: Expr = Expr::Natural(Natural(("0",)));
