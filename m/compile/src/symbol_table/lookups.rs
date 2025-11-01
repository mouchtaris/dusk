use super::*;

pub type NameMatch = fn(/*name*/ &str, /*var*/ &str) -> bool;

pub fn last_of_path_match() -> NameMatch {
    |name, var| {
        if var.len() >= name.len() + 1 {
            // +1 because '.../<name>'
            let (_, suffix) = var.split_at(var.len() - name.len());

            if suffix.starts_with('/') && suffix.ends_with(name) {
                return true;
            }
        }

        false
    }
}

pub fn default_auto() -> impl MatchScope {
    |name, scope| Ok(te!(scope.lookup_by_name(name), "lookup-default-auto"))
}

type X<'a> = &'a SymID;

pub trait Matcher<T, U>: Fn(&str, T) -> U {
    fn bind_name(&self, name: &str) -> impl Fn(T) -> U {
        |x| self(name, x)
    }
    fn pick_right(&self, next: impl Matcher<T, U>) -> impl Matcher<T, U> {
        next
    }
}
impl<S: Fn(&str, T) -> U, T, U> Matcher<T, U> for S {}
pub fn matcher<T, U, X: Matcher<T, U>>(x: X) -> X {
    x
}
pub fn match_scope<X: MatchScope>(x: X) -> X {
    x
}

pub trait MatchName: for<'r> Matcher<&'r str, bool> {
    fn to_match_symbol(&self) -> impl MatchSymbol {
        |name, (var, sym_id)| {
            if self(name, var) {
                Some(sym_id)
            } else {
                None
            }
        }
    }
}
impl<S: for<'r> Matcher<&'r str, bool>> MatchName for S {}

pub trait MatchSymbol: for<'r, 'v> Matcher<(&'v str, &'r SymID), Option<X<'r>>> {
    fn to_match_scope(&self) -> impl MatchScope {
        |name, scope| {
            if let Some(x) = scope.symbols().find_map(self.bind_name(name)) {
                return Ok(x);
            }
            temg!("not found (match-symbol)")
        }
    }
}
impl<S: for<'r, 'v> Matcher<(&'v str, &'r SymID), Option<X<'r>>>> MatchSymbol for S {}

pub trait MatchScope: for<'r> Matcher<&'r Scope, Result<X<'r>>> {
    fn or_else(&self, next: impl MatchScope) -> impl MatchScope {
        move |name, scope| self(name, scope).or_else(|_| next(name, scope))
    }

    fn to_match_scopes<Scopes: ?Sized + ScopesRef>(&self) -> impl MatchScopes<Scopes> {
        |name, scopes| {
            lookup_by_pred_in_scopes(
                "(match-scope)",
                |scope| self(name, scope),
                scopes.active_scopes(),
            )
        }
    }
}
impl<S: for<'r> Matcher<&'r Scope, Result<X<'r>>>> MatchScope for S {}

pub trait MatchScopes<Scopes: ?Sized + ScopesRef>:
    for<'r> Matcher<&'r Scopes, Result<X<'r>>>
{
}
impl<Scopes: ?Sized + ScopesRef, S: for<'r> Matcher<&'r Scopes, Result<X<'r>>>> MatchScopes<Scopes>
    for S
{
}

pub trait Match {
    type T;
    type U;
    fn apply(&self, name: &str, data: Self::T) -> Self::U;
}
