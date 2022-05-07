use {
    super::{sym, te, temg, Deq, Map, Result},
    error::ltrace,
    std::{borrow::Borrow, fmt},
};

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub(crate) scopes: Scopes,
    pub(crate) scope_stack: ScopeStack,
}

pub type ScopeStack = Deq<usize>;
pub type Scope = Map<String, SymInfo>;
pub type Scopes = Vec<Scope>;
pub type SymInfo = sym::Info;
pub type SymType = sym::Typ;

pub trait SymbolTableExt
where
    Self: AsRef<SymbolTable> + AsMut<SymbolTable>,
{
    fn new_array<T, N>(&mut self, types: T, name: N) -> SymInfo
    where
        T: IntoIterator,
        T::Item: Into<SymInfo>,
        N: Into<String>,
    {
        let mut sinfo = SymInfo::typ(sym::Typ::array(types));
        sinfo.scope_id = self.scope_id();
        self.scope_mut().insert(name.into(), sinfo.to_owned());
        sinfo
    }

    fn new_address<S: Into<String>>(&mut self, name: S, addr: usize, ret_t: &SymInfo) -> SymInfo {
        let scope_id = self.scope_id();
        let scope = self.scope_mut();
        let sinfo = SymInfo {
            scope_id,
            typ: SymType::address(addr, ret_t),
        };
        let name = name.into();
        scope.insert(name, sinfo.to_owned());
        sinfo
    }

    fn new_local<T>(&mut self, types: T, name: String) -> &mut SymInfo
    where
        T: IntoIterator,
        T::Item: Into<SymInfo>,
    {
        let scope_id = self.scope_id();
        let scope = self.scope_mut();
        let local_var = sym::Local {
            fp_off: scope_stack_size(&scope),
            is_alias: false,
            types: types.into_iter().map(<_>::into).collect(),
        };
        let sinfo = SymInfo {
            scope_id,
            typ: SymType::Local(local_var),
        };
        scope
            .entry(name)
            .and_modify(|i| *i = sinfo.to_owned())
            .or_insert(sinfo)
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
        if cfg!(feature = "debug") {
            callb(&format_args!(
                "t:{}:{}:{}",
                self.scope_id(),
                self.scope().len(),
                name
            ))
        } else {
            callb(&format_args!("{}:{}", self.scope_id(), self.scope().len()))
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
        syminfo.typ = sym::Typ::Literal(sym::Literal {
            id: nat,
            lit_type: sym::LitType::Natural,
        });
        syminfo
    }

    fn alias_name<S: Into<String>>(&mut self, new_name: S, info: &SymInfo) {
        let st = self.as_mut();
        let scope = &mut st.scopes[info.scope_id];

        let new_name = new_name.into();
        let mut info = info.to_owned();
        match &mut info.typ {
            SymType::Local(sym::Local { is_alias, .. }) => *is_alias = true,
            _ => (),
        }
        scope.insert(new_name, info);
    }

    fn lookup<S>(&self, name: S) -> Result<&SymInfo>
    where
        S: Borrow<str>,
    {
        let name = name.borrow();
        let sym_table = self.as_ref();

        for &scope_id in &sym_table.scope_stack {
            let scope = &sym_table.scopes[scope_id];
            match scope.get(name) {
                Some(sinfo) => return Ok(sinfo),
                _ => (),
            }
        }
        temg!("Symbol not found: {}", name)
    }
    fn lookup_var<S>(&self, name: S) -> Result<&sym::Local>
    where
        S: Borrow<str>,
    {
        self.lookup(name).and_then(|i| i.as_local_ref())
    }
    fn lookup_name(&self, sinfo: &SymInfo) -> Result<&str> {
        let sym_table = self.as_ref();

        let scope_id = sinfo.scope_id;
        let scope = te!(sym_table.scopes.get(scope_id), "Scope id: {:?}", scope_id);

        for (name, info) in scope {
            if info == sinfo {
                return Ok(name);
            }
        }
        temg!("Name not found: {:?}", sinfo)
    }

    fn enter_scope(&mut self) {
        let sym_table = self.as_mut();
        sym_table.scopes.push(<_>::default());

        let id = sym_table.scopes.len() - 1;
        sym_table.scope_stack.push_front(id);

        ltrace!("scope::enter {}", self.scope_id());
    }

    fn exit_scope(&mut self) {
        let sym_table = self.as_mut();
        sym_table.scope_stack.pop_front();

        ltrace!("scope::exit {}", self.scope_id());
    }

    /// Current scope id
    fn scope_id(&self) -> usize {
        let sym_table = self.as_ref();
        sym_table.scope_stack.front().cloned().unwrap_or(0)
    }

    fn next_scope_id(&mut self) -> usize {
        let sym_table = self.as_mut();
        sym_table.scopes.len()
    }

    /// Get the current scope
    fn scope(&self) -> &Scope {
        let sym_table = self.as_ref();
        &sym_table.scopes[sym_table.scope_id()]
    }

    fn scope_mut(&mut self) -> &mut Scope {
        let sym_table = self.as_mut();
        let id = sym_table.scope_id();
        &mut sym_table.scopes[id]
    }

    fn stack_frame_size(&self) -> usize {
        let sym_table = self.as_ref();
        scope_stack_size(sym_table.scope())
    }
}

pub fn scope_stack_size(scope: &Scope) -> usize {
    scope
        .iter()
        .filter_map(|(_, info)| info.as_local_ref().ok().filter(|i| !i.is_alias))
        .count()
}

impl SymbolTableExt for SymbolTable {}
impl AsRef<SymbolTable> for SymbolTable {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl AsMut<SymbolTable> for SymbolTable {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub fn scopes<'s, S>(st: &'s S) -> impl ExactSizeIterator<Item = (usize, &'s str, &'s SymInfo)>
where
    S: AsRef<SymbolTable>,
{
    st.as_ref()
        .scopes
        .iter()
        .enumerate()
        .flat_map(move |(scope_id, scope)| {
            scope
                .iter()
                .map(move |(name, info)| (scope_id, name.as_str(), info))
        })
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn find_func_name<'s, S: AsRef<SymbolTable>>(st: &'s S, faddr: &usize) -> Option<&'s str> {
    scopes(st).find_map(|(_, n, i)| match i {
        sym::Info {
            typ: sym::Typ::Address(sym::Address { addr, .. }),
            ..
        } if addr == faddr => Some(n),
        _ => None,
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
