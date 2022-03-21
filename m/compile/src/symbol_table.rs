use {
    super::{sym, Map},
    std::{borrow::Borrow, fmt},
};

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub(crate) scopes: Vec<Scope>,
    pub(crate) scope_stack: Vec<usize>,
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
        };
        let name = name.into();
        scope.insert(name, sinfo.clone());
        sinfo
    }

    fn new_local(&mut self, name: String) -> SymInfo {
        let scope_id = self.scope_id();
        let scope = self.scope_mut();
        let fp_off = scope.len();
        let sinfo = SymInfo {
            scope_id,
            typ: SymType::Local(sym::Local { fp_off }),
        };
        scope.insert(name, sinfo.clone());
        sinfo
    }

    fn new_local_tmp<D>(&mut self, desc: D) -> SymInfo
    where
        D: fmt::Display,
    {
        let name = format!("t:{}:{}:{}", self.scope_id(), self.scope().len(), desc);
        self.new_local(name)
    }

    fn lookup<S>(&self, name: S) -> Option<&SymInfo>
    where
        S: Borrow<str>,
    {
        let sym_table = self.as_ref();
        for &scope_id in &sym_table.scope_stack {
            let scope = &sym_table.scopes[scope_id];
            match scope.get(name.borrow()) {
                sinfo @ Some(_) => return sinfo,
                _ => (),
            }
        }
        None
    }
    fn lookup_addr<S>(&self, name: S) -> Option<&sym::Address>
    where
        S: Borrow<str>,
    {
        self.lookup(name).and_then(|i| i.as_addr_ref())
    }

    fn enter_scope(&mut self) {
        let sym_table = self.as_mut();
        sym_table.scopes.push(<_>::default());

        let id = sym_table.scopes.len() - 1;
        sym_table.scope_stack.push(id);
    }

    fn exit_scope(&mut self) {
        let sym_table = self.as_mut();
        sym_table.scope_stack.pop();
    }

    /// Current scope id
    fn scope_id(&self) -> usize {
        let sym_table = self.as_ref();
        sym_table.scope_stack.last().cloned().unwrap_or(0)
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
        sym_table
            .scope()
            .iter()
            .filter_map(|(_, info)| info.as_local_ref().ok())
            .count()
    }
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
