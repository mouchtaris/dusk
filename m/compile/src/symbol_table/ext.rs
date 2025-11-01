use super::*;

impl<S: scope::ApiRef> ScopeRef for S {}
impl<S: scope::ApiMut> ScopeMut for S {}

impl<S: ?Sized + scopes::ApiMut + scopes::ApiRef> ScopesExt for S {}
impl<S: ?Sized + scopes::ApiRef> ScopesRef for S {}

impl<S: Into<String>> ToName for S {}
impl<S: ToOwned<Owned = SymInfo>> ToSymInfo for S {}

trait ToName: Into<String> {}
trait ToSymInfo: ToOwned<Owned = SymInfo> {}

use scope::{ApiMut as ScopeApiMut, ApiRef as ScopeApiRef, SymDetails};

// ----------------------------------------------------------------------------
// Scope Ext
pub(crate) trait ScopeRef: scope::ApiRef {}

pub(crate) trait ScopeMut: scope::ApiMut {}

// ----------------------------------------------------------------------------
// Scopes Ext
//
pub(crate) trait ScopesRef: scopes::ApiRef {
    fn current_scope_id(&self) -> usize {
        scopes::ApiRef::scope_id(self)
    }

    // Clients can inspect scopes for custom deductions
    // (ex: stack size based on locals)
    fn scope(&self) -> &impl ScopeRef {
        let current = self.scope_id();
        self.get_scope(current)
    }

    /// The id for the next symbol within the current scope
    fn next_symbol_id(&self) -> usize {
        self.scope().next_id()
    }

    /// The pinnacle of reverse-symbol-lookup: *active_symbols* returns
    /// each active symbol in the current scope, in reverse order of
    /// appearance.
    ///
    /// This means that most recently added symbols are returned first,
    /// and most long-ago added symbols are returned last.
    ///
    /// This can be (is) used to allow referring to the "closest" symbols
    /// at the current scope point.
    fn active_symbols_in_reverse(&self) -> impl Iterator<Item = (&str, &SymID)> {
        self.active_scope_stack()
            .flat_map(|scope| scope.all_symbols_in_reverse())
    }
}

pub(crate) trait ScopesExt: scopes::ApiMut + scopes::ApiRef {
    fn alias_in_scope(&mut self, info: &SymInfo, new_name: impl ToName) -> SymID {
        let (_, info, _) = self.insert_to_scope_details(new_name, info.aliased());
        info.to_owned()
    }

    fn insert_to_scope(&mut self, name: impl ToName, info: impl ToSymInfo) -> SymInfo {
        self.insert_to_scope_mut(name, info).to_owned()
    }

    fn insert_to_scope_mut(&mut self, name: impl ToName, info: impl ToSymInfo) -> &mut SymInfo {
        let (_, sym_id, _) = self.insert_to_scope_details(name, info);
        sym_id.sym_info_mut()
    }

    /// Return
    /// - `&str` name
    /// - `&mut SymID`: (sym_id * sym_info)
    /// - `usize` prev sym_id (as usize because can't borrow all together)
    fn insert_to_scope_details(&mut self, name: impl ToName, info: impl ToSymInfo) -> SymDetails {
        let scope_id = self.scope_id();
        self.scope_mut()
            .insert(name.into(), info.to_owned().in_scope(scope_id))
    }
}

impl<S: ?Sized + ScopesExt> ScopesExtPrivate for S {}
impl<S: ?Sized + ScopesRef> ScopesRefPrivate for S {}

trait ScopesExtPrivate: ScopesExt {
    fn scope_mut(&mut self) -> &mut impl ScopeMut {
        let id = self.scope_id();
        self.get_scope_mut(id)
    }

    fn get_scope_mut(&mut self, scope_id: usize) -> &mut impl ScopeMut {
        self.scopes_mut().nth(scope_id).expect("scope")
    }
}

trait ScopesRefPrivate: ScopesRef {
    fn get_scope(&self, scope_id: usize) -> &impl ScopeRef {
        self.scopes().nth(scope_id).expect("scope")
    }
}
