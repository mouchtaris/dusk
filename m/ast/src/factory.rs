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
