use super::*;

impl<S: ?Sized + Ref<Scope>> ScopeRef for S {}
impl<S: ?Sized + Mut<Scope>> ScopeMut for S {}

impl<S: ?Sized + scopes::ApiMut + scopes::ApiRef> ScopesExt for S {}
impl<S: ?Sized + scopes::ApiRef> ScopesRef for S {}

impl<S: Into<String>> ToName for S {}
impl<S: ToOwned<Owned = SymInfo>> ToSymInfo for S {}

pub trait ToName: Into<String> {}
pub trait ToSymInfo: ToOwned<Owned = SymInfo> {}

use scope::{ApiMut as ScopeApiMut, ApiRef as ScopeApiRef, SymDetails};

// ----------------------------------------------------------------------------
// Scope Ext
pub trait ScopeRef: Ref<Scope> {
    fn lookup_by_name(&self, name: impl Ref<str>) -> Option<&SymID> {
        ScopeApiRef::lookup_by_name(self, name)
    }

    #[deprecated(note = "use lookup_by_name with SymID")]
    fn symbol_name_from_info_alone(&self, info: &SymInfo) -> Option<&str> {
        ScopeApiRef::sym_name_from_info(self, info)
    }

    /// All symbols in the scope, in reverse order of appearance
    /// (most recent to most old)
    fn symbols(&self) -> impl Seq<Item = (&str, &SymID)> {
        ScopeApiRef::all_symbols_in_reverse(self)
    }
}

pub trait ScopeMut: Mut<Scope> {}

// ----------------------------------------------------------------------------
// Scopes Ext
//
pub(crate) trait ScopesRef: scopes::ApiRef {
    /// List all scopes ever defined, in order of appearance.
    fn list_all_scopes(&self) -> impl Seq<Item = &impl ScopeRef> {
        self.all_scopes()
    }

    fn get_scope(&self, scope_id: usize) -> Option<&impl ScopeRef> {
        self.list_all_scopes().nth(scope_id)
    }

    fn current_scope_id(&self) -> usize {
        scopes::ApiRef::scope_id(self)
    }

    // Clients can inspect scopes for custom deductions
    // (ex: stack size based on locals)
    fn current_scope(&self) -> &impl ScopeRef {
        let current = self.scope_id();
        self.get_scope(current).expect("current scope")
    }

    /// The id for the next symbol within the current scope
    fn next_symbol_id(&self) -> usize {
        self.current_scope().next_id()
    }

    /// Active scopes, inner-to-outer.
    ///
    /// The pinnacle of reverse-scope-based lookups.
    ///
    fn active_scopes(&self) -> impl Seq<Item = &impl ScopeRef> {
        self.active_scope_stack()
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
        self.active_scopes()
            .flat_map(|scope| scope.all_symbols_in_reverse())
    }

    /// All symbols ever defined, in all scopes, active and hidden.
    fn all_symbols(&self) -> impl Iterator<Item = (&str, &SymID)> {
        self.all_scopes()
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

trait ScopesRefPrivate: ScopesRef {}
