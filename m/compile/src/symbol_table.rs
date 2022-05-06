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
    fn new_address<S: Into<String>>(&mut self, name: S, addr: usize, retval_size: u16) -> SymInfo {
        let scope_id = self.scope_id();
        let scope = self.scope_mut();
        let sinfo = SymInfo {
            scope_id,
            typ: SymType::Address(sym::Address { addr, retval_size }),
        };
        let name = name.into();
        scope.insert(name, sinfo.clone());
        sinfo
    }

    fn new_local(&mut self, name: String) -> &mut SymInfo {
        let scope_id = self.scope_id();
        let scope = self.scope_mut();
        let local_var = sym::Local {
            fp_off: scope_stack_size(&scope),
            is_alias: false,
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

    fn new_local_tmp<D>(&mut self, desc: D) -> &mut SymInfo
    where
        D: fmt::Display,
    {
        let name = if cfg!(feature = "debug") {
            format!("t:{}:{}:{}", self.scope_id(), self.scope().len(), desc)
        } else {
            format!("{}:{}", self.scope_id(), self.scope().len())
        };
        self.new_local(name)
    }

    fn new_natural_literal_tmp(&mut self, nat: usize) -> &mut SymInfo {
        let syminfo = self.new_local_tmp(format_args!("literal-nat-{}", nat));
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
        let mut info = info.clone();
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
