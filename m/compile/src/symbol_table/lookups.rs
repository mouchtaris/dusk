use super::*;

/// The main entry-point for auto-name lookups.
pub fn lookup_auto(this: &(impl ?Sized + ScopesRef), name: impl Ref<str>) -> Result<&SymID> {
    let name = name.borrow();

    use lookups::*;

    let flow = default_auto();
    let flow = flow.to_match_scopes();

    #[cfg(feature = "funny_name_lookup")]
    let flow = match_scopes(|name, x| {
        let next = last_of_path_match("/");
        let next = next.to_match_symbol();
        let next = next.to_match_scope();
        let next = next.to_match_scopes();
        let flow = flow.or_else(next);

        let next = last_of_path_match("::");
        let next = next.to_match_symbol();
        let next = next.to_match_scope();
        let next = next.to_match_scopes();
        let flow = flow.or_else(next);

        let x = flow(name, x);
        x
    });

    let flow = flow;

    let sym_id = flow(name, this);

    sym_id
}

pub fn last_of_path_match(sep: &str) -> impl '_ + MatchName {
    use std::eprintln as db;
    //use std::format_args as db;
    move |name, var| {
        db!("Checking {name} {sep} {var}: ");
        if var.len() >= name.len() + sep.len() {
            // +1 because '.../<name>'
            let (_, suffix) = var.split_at(var.len() - name.len() - sep.len());

            if suffix.starts_with(sep) && suffix.ends_with(name) {
                db!("IT IS!");
                return true;
            } else {
                db!(
                    "Suffix not ok: {} starts_with '{}' && ends_with {}",
                    suffix,
                    sep,
                    name
                );
            }
        } else {
            db!(
                "Length not ok: {} >= {} + {}",
                var.len(),
                name.len(),
                sep.len()
            );
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
pub fn match_scopes<S: ?Sized + ScopesRef, X: MatchScopes<S>>(x: X) -> X {
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
                format!("(match-scope): {name}"),
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
    fn or_else(&self, next: impl MatchScopes<Scopes>) -> impl MatchScopes<Scopes> {
        move |name, scopes| self(name, scopes).or_else(|_| next(name, scopes))
    }
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

#[test]
#[cfg(feature = "funny_name_lookup")]
fn test_funny_names_lookup() -> Result<()> {
    let mut cmp = crate::Compiler::new();

    let st = &mut cmp.sym_table;

    use std::format as f;

    fn empty() -> impl Iterator<Item = SymInfo> {
        std::iter::empty()
    }
    st.enter_scope();
    let info_0 = st.new_local(empty(), f!("a")).to_owned();
    let info_1 = st.new_local(empty(), f!("a/b")).to_owned();
    let info_2 = st.new_local(empty(), f!("a/b/c")).to_owned();

    let info = te!(lookup_auto(st, "a"));
    assert_eq!(info.sym_info(), &info_0);

    let info = te!(lookup_auto(st, "a/b"));
    assert_eq!(info.sym_info(), &info_1);

    let info = te!(lookup_auto(st, "a/b/c"));
    assert_eq!(info.sym_info(), &info_2);

    // FUNNY NAME LOOKUP
    let info = te!(lookup_auto(st, "c"));
    assert_eq!(info.sym_info(), &info_2);

    let scope_id = st.enter_scope();

    let info_3 = st.new_local(empty(), f!("a/b/c/a")).to_owned();

    let info = te!(lookup_auto(st, "a"));
    assert_eq!(info.sym_info(), &info_0);

    let info = te!(lookup_auto(st, "c/a"));
    assert_eq!(info.sym_info(), &info_3);
    assert_eq!(info.sym_info().scope_id, scope_id);

    let info_5 = st.new_local(empty(), f!("a::b")).to_owned();
    assert_eq!(te!(lookup_auto(st, "b")).sym_info(), &info_1);
    assert_eq!(te!(lookup_auto(st, "a::b")).sym_info(), &info_5);

    let info_6 = st.new_local(empty(), f!("a::d")).to_owned();
    assert_eq!(te!(lookup_auto(st, "d")).sym_info(), &info_6);

    Ok(())
}
