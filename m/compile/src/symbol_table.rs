use {
    super::{fmt, sym, te, temg, Deq, Map, Mut, Ref, Result, Seq},
    error::{ltrace, IntoResult},
};

mod ext;
#[cfg(feature = "funny_name_lookup")]
mod funny_name_lookup;
pub mod lookups;
mod scope;
mod scopes;

pub use {
    ext::{ScopeMut, ScopeRef, ToName},
    scopes::SymbolTable,
};

pub(crate) use {
    ext::{ScopesExt, ScopesRef},
    scope::{Scope, SymID},
};

pub type SymInfo = sym::Info;

impl<S: Ref<SymbolTable> + Mut<SymbolTable>> SymbolTableExt for S {}
pub trait SymbolTableExt
where
    Self: Ref<SymbolTable> + Mut<SymbolTable>,
{
    fn new_array<T, N>(&mut self, types: T, name: N) -> SymInfo
    where
        T: IntoIterator,
        T::Item: Into<SymInfo>,
        N: Into<String>,
    {
        self.insert_to_scope(name, SymInfo::typ(sym::Typ::array(types)))
    }

    fn new_address<S: Into<String>>(&mut self, name: S, addr: usize, ret_t: &SymInfo) -> SymInfo {
        self.insert_to_scope(name, SymInfo::address(addr, ret_t))
    }

    fn new_local<T>(&mut self, types: T, name: String) -> &mut SymInfo
    where
        T: IntoIterator,
        T::Item: Into<SymInfo>,
    {
        let fp_off = self.stack_frame_size();
        self.insert_to_scope_mut(name, SymInfo::local(fp_off, types))
    }

    fn with_tmp_name<D, F>(&mut self, name: D, func: F) -> &mut SymInfo
    where
        D: fmt::Display,
        F: FnOnce(&mut Self, String) -> &mut SymInfo,
    {
        func(self, self.tmp_name_string(name))
    }

    fn tmp_name_string<D: fmt::Display>(&self, name: D) -> String {
        self.tmp_name(name, |name| name.to_string())
    }

    fn tmp_name<D, K, R>(&self, name: D, callb: K) -> R
    where
        D: fmt::Display,
        K: FnOnce(&dyn fmt::Display) -> R,
    {
        let scope_id = self.current_scope_id();
        let tmp_id = self.next_symbol_id();

        if cfg!(feature = "debug") {
            callb(&format_args!("t:{scope_id}:{tmp_id}:{name}",))
        } else {
            callb(&format_args!("{scope_id}:{tmp_id}"))
        }
    }

    fn new_local_tmp2<T, D>(&mut self, types: T, desc: D) -> &mut SymInfo
    where
        T: IntoIterator,
        T::Item: Into<SymInfo>,
        D: fmt::Display,
    {
        self.with_tmp_name(desc, |st, name| st.new_local(types, name))
    }

    fn new_local_tmp<T, D>(&mut self, types: T, desc: D) -> &mut SymInfo
    where
        T: TypesMagnet,
        D: fmt::Display,
    {
        self.new_local_tmp2(types.magnetize_to_types(), desc)
    }

    fn new_natural_literal_tmp(&mut self, nat: usize) -> &mut SymInfo {
        let syminfo = self.new_local_tmp(
            SymInfo::lit_natural(nat),
            format_args!("literal-nat-{}", nat),
        );
        // TODO why set it again -- might be redundant
        syminfo.typ = sym::Typ::Literal(sym::Literal {
            id: nat,
            lit_type: sym::LitType::Natural,
        });
        syminfo
    }

    #[deprecated(note = "Use `alias_in_scope`")]
    fn alias_name<S: Into<String>>(&mut self, new_name: S, info: &SymInfo) {
        self.alias_in_scope(info, new_name);
    }
    fn alias_in_scope(&mut self, info: &SymInfo, new_name: impl ToName) {
        ScopesExt::alias_in_scope(self, info, new_name);
    }

    /// Lookup by exact name in all scopes.
    fn lookup_by_name_everywhere(&self, name: impl Ref<str>) -> Result<&SymID> {
        Ok(te!(lookup_by_name_in_scopes(name, self.list_all_scopes())))
    }

    /// Lookup by exact name in active scopes.
    fn lookup<S>(&self, name: S) -> Result<&SymInfo>
    where
        S: Ref<str>,
    {
        let this = self;
        let name = name.borrow();

        fn pred<X: Fn(&Scope) -> Result<&SymID>>(x: X) -> X {
            x
        }

        let exact_name = pred(|scope| scope.lookup_by_name(name).into_result());
        #[cfg(feature = "funny_name_lookup")]
        let last_of_slash_path = pred(|scope| funny_name_lookup::accept(name)(scope));
        #[cfg(not(feature = "funny_name_lookup"))]
        let last_of_slash_path = pred(|_| false);

        type X<'a> = &'a SymInfo;
        type R<'a> = Result<X<'a>>;

        fn stage<'a, I, X: Fn(I) -> R<'a>>(x: X) -> X {
            x
        }

        type Stage<'a, I> = fn(I) -> Result<X<'a>>;
        fn z<'a, I>() -> Stage<'a, I> {
            |_| Err("").into_result()
        }

        let a = stage(|_| Ok(te!(lookup_by_name_in_scopes(name, this.active_scopes())).sym_info()));
        #[cfg(feature = "funny_name_lookup")]
        let b = stage(|_| Ok(te!(funny_name_lookup::lookup(this, name))));
        #[cfg(not(feature = "funny_name_lookup"))]
        let b = stage(z());

        let w0 = {
            let c = stage(z());
            let z = stage(z());
            let z = z(());

            c(z).or_else(b).or_else(a)
        };

        let w1 = {};

        w0
    }
    /// Lookup by [SymbolTableExt::lookup] and ensure it's a local symbol.
    fn lookup_var<S>(&self, name: S) -> Result<&sym::Local>
    where
        S: Ref<str>,
    {
        self.lookup(name).and_then(|i| i.as_local_ref())
    }
    #[deprecated(note = "use ScopesRef::symbol_name() with SymID")]
    fn lookup_name(&self, sinfo: &SymInfo) -> Result<&str> {
        let scope_id = sinfo.scope_id;
        let scope = te!(self.get_scope(scope_id), "Scope id: {:?}", scope_id);
        if let Some(name) = scope.symbol_name_from_info_alone(sinfo) {
            return Ok(name);
        }
        temg!("Name not found: {:?}", sinfo)
    }

    fn enter_scope(&mut self) -> usize {
        scopes::ApiMut::enter_scope(self)
    }

    fn exit_scope(&mut self) {
        scopes::ApiMut::exit_scope(self)
    }

    /// Get the current scope
    fn current_scope(&self) -> &impl ScopeRef {
        ScopesRef::current_scope(self)
    }

    fn stack_frame_size(&self) -> usize {
        scope_stack_size(self.current_scope())
    }

    /// Global scope is scope id *`1`* !!
    ///
    /// Scope `0` is the system scope. Although these are runtime consideerations,
    /// they're somehow intertwined here.
    fn global_scope_opt(&self) -> Option<&impl ScopeRef> {
        self.get_scope(1)
    }
}

/// Lookup in the given scopes, in order, for the first `pred(..) == true`.
pub fn lookup_by_pred_in_scopes<'a>(
    info: impl fmt::Display,
    pred: impl Fn(&Scope) -> Result<&SymID>,
    mut scopes: impl Iterator<Item = &'a (impl ScopeRef + 'a)>,
) -> Result<&'a SymID> {
    if let Some(sid) = scopes.find_map(|scope| pred(scope.borrow()).ok()) {
        return Ok(sid);
    }
    temg!("Symbol not found: {}", info)
}

/// Lookup in the given scopes, in order, for an exact name match.
pub fn lookup_by_name_in_scopes<'a>(
    name: impl Ref<str>,
    scopes: impl Iterator<Item = &'a (impl ScopeRef + 'a)>,
) -> Result<&'a SymID> {
    let name = name.borrow();

    lookup_by_pred_in_scopes(
        name,
        |scope| ScopeRef::lookup_by_name(scope, name).into_result(),
        scopes,
    )
}

/// Stack size requirement, according to recorded locals.
///
/// At the moment, each local counts as 1 cell.
///
pub fn scope_stack_size(scope: &(impl ScopeRef + ?Sized)) -> usize {
    scope
        .symbols()
        .filter_map(|(_, info)| info.sym_info().as_local_ref().ok().filter(|i| !i.is_alias))
        .count()
}

/// Find a function name by function address.
///
/// Looks-through *all* symbols.
pub fn find_func_name<'s>(st: &'s impl SymbolTableExt, faddr: &usize) -> Option<&'s str> {
    st.all_symbols().find_map(|(n, i)| {
        let sym::Address { addr, .. } = i.sym_info().as_addr_ref().ok()?;
        if addr == faddr {
            return Some(n);
        }
        None
    })
}

pub trait TypesMagnet {
    fn magnetize_to_types(self) -> Self::Types;

    type Types: IntoIterator<Item = Self::Item>;
    type Item: Into<SymInfo>;
}

impl<S> TypesMagnet for S
where
    S: IntoIterator,
    S::Item: Into<SymInfo>,
{
    fn magnetize_to_types(self) -> Self {
        self
    }

    type Types = Self;
    type Item = <Self as IntoIterator>::Item;
}
macro_rules! types_magnet_as {
    ($a:ty $([$($lt:lifetime),*])?, $b:ty, $c:expr) => {
        impl
            $(<$($lt),*>)?
        TypesMagnet for $a
        {
            type Types = <$b as TypesMagnet>::Types;
            type Item = <$b as TypesMagnet>::Item;
            fn magnetize_to_types(self) -> Self::Types { $c(self).magnetize_to_types() }
        }
    }
}
types_magnet_as!(SymInfo, [SymInfo; 1], |s| [s]);
types_magnet_as!(&'a SymInfo['a], SymInfo, <_>::to_owned);
