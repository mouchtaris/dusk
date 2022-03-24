use {
    super::{sym, temg, Deq, Map, Result, Type},
    std::{borrow::Borrow, fmt},
};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SymbolTable {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) scope_stack: Deq<usize>,
}

pub type Scope = Map<String, SymInfo>;
pub type SymInfo = sym::Info;
pub type SymType = sym::Typ;

pub trait SymbolTableExt
where
    Self: AsRef<SymbolTable> + AsMut<SymbolTable>,
{
    fn new_address<S: Into<String>>(&mut self, name: S, addr: usize) -> SymInfo {
        let scope_id = self.scope_id();
        let scope = self.scope_mut();
        let sinfo = SymInfo {
            scope_id,
            typ: SymType::Address(sym::Address { addr }),
            static_type: Type::any(),
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
        let mut sinfo = SymInfo {
            scope_id,
            typ: SymType::Local(local_var),
            static_type: Type::any(),
        };
        scope.entry(name).or_insert(sinfo)
    }

    fn new_local_tmp<D>(&mut self, desc: D) -> &mut SymInfo
    where
        D: fmt::Display,
    {
        let name = format!("t:{}:{}:{}", self.scope_id(), self.scope().len(), desc);
        self.new_local(name)
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
    fn lookup_addr<S>(&self, name: S) -> Result<&sym::Address>
    where
        S: Borrow<str>,
    {
        self.lookup(name).and_then(|i| i.as_addr_ref())
    }

    fn enter_scope(&mut self) {
        let sym_table = self.as_mut();
        sym_table.scopes.push(<_>::default());

        let id = sym_table.scopes.len() - 1;
        sym_table.scope_stack.push_front(id);
    }

    fn exit_scope(&mut self) {
        let sym_table = self.as_mut();
        sym_table.scope_stack.pop_front();
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
