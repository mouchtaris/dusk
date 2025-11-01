use {
    super::{fmt, sym, te, temg, Deq, Map, Mut, Ref, Result, Seq},
    error::ltrace,
};

mod ext;
mod scope;
mod scopes;

pub(super) use {
    ext::{ScopeMut, ScopeRef, ScopesExt, ScopesRef, ToName},
    scope::{Scope, SymID},
    scopes::SymbolTable,
};

pub type SymInfo = sym::Info;
pub type SymType = sym::Typ;

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

    /// Lookup by exact name in active scopes.
    fn lookup<S>(&self, name: S) -> Result<&SymInfo>
    where
        S: Ref<str>,
    {
        let name = name.borrow();

        for scope in self.active_scopes() {
            if let Some(sid) = scope.lookup_by_name(name) {
                return Ok(sid.sym_info());
            }
        }

        temg!("Symbol not found: {}", name)
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
    fn scope(&self) -> &impl ScopeRef {
        ScopesRef::current_scope(self)
    }

    fn stack_frame_size(&self) -> usize {
        scope_stack_size(self.scope())
    }
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
pub fn find_func_name<'s, S: AsRef<SymbolTable>>(st: &'s S, faddr: &usize) -> Option<&'s str> {
    st.as_ref().all_symbols().find_map(|(n, i)| {
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
