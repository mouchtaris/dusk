use super::*;

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    scopes: Scopes,
    scope_stack: ScopeStack,
}
buf::sd_struct![SymbolTable, scopes, scope_stack];

impl<S: ?Sized + Ref<SymbolTable>> ApiRef for S {}
impl<S: ?Sized + Mut<SymbolTable>> ApiMut for S {}

pub(super) trait ApiRef: Ref<SymbolTable> {
    fn scope_id(&self) -> usize {
        let (_, stack, ..) = parts(self);

        stack.front().cloned().unwrap_or(0)
    }

    /// The id for the next scope
    fn next_scope_id(&self) -> usize {
        let (scopes, ..) = parts(self);
        scopes.len()
    }

    // Clients can inspect scopes for custom deductions
    // (ex: stack size based on locals)
    /// All scopes, in order of appearance
    fn all_scopes(&self) -> impl Seq<Item = &impl ScopeRef> {
        let (scopes, ..) = parts(self);
        scopes.iter()
    }

    // basic backbone of lookups: lookup in reverse enclosing scopes
    /// Active scopes, inner-to-outer
    fn active_scope_stack(&self) -> impl Seq<Item = &impl ScopeRef> {
        let (scopes, stack, ..) = parts(self);
        stack.iter().rev().copied().map(|id| &scopes[id])
    }
}

pub(super) trait ApiMut: Mut<SymbolTable> {
    /// All scopes -- used for indexed access
    fn scopes_mut(&mut self) -> impl Seq<Item = &mut impl scope::ApiMut> {
        let (scopes, ..) = parts_mut(self);
        scopes.iter_mut()
    }

    fn enter_scope(&mut self) -> usize {
        let id = self.next_scope_id();

        let (scopes, stack, ..) = parts_mut(self);

        scopes.push(<_>::default());
        stack.push_front(id);

        ltrace!("scope::enter {}", self.scope_id());

        id
    }
}

macro_rules! invariants {
    ($this:expr) => {{
        let this = $this;

        let SymbolTable {
            scopes,
            scope_stack,
            ..
        } = &*this;

        assert!(scopes.len() == scope_stack.len());

        this
    }};
}
fn parts(this: &(impl Ref<SymbolTable> + ?Sized)) -> (&Scopes, &ScopeStack) {
    let SymbolTable {
        scopes,
        scope_stack,
        ..
    } = invariants!(this.borrow());
    (scopes, scope_stack)
}
fn parts_mut(this: &mut (impl Mut<SymbolTable> + ?Sized)) -> (&mut Scopes, &mut ScopeStack) {
    let SymbolTable {
        scopes,
        scope_stack,
        ..
    } = invariants!(this.borrow_mut());
    (scopes, scope_stack)
}
fn as_ref(this: &mut (impl Mut<SymbolTable> + ?Sized)) -> &impl ApiRef {
    invariants!(this.borrow_mut())
}
