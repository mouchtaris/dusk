use {
    super::{temg, Result},
    std::fmt,
};

#[derive(Clone, Eq, PartialEq)]
pub struct Info {
    pub typ: Typ,
    pub scope_id: usize,
}

either::either![
    #[derive(Clone, Eq, PartialEq)]
    pub Typ,
        Local,
        Address,
        Literal
];
#[derive(Clone, Eq, PartialEq)]
pub struct Local {
    pub fp_off: usize,
    pub is_alias: bool,
    pub types: Vec<Info>,
}
#[derive(Clone, Eq, PartialEq)]
pub struct Address {
    pub addr: usize,
    pub ret_t: Box<Info>,
}
#[derive(Clone, Eq, PartialEq)]
pub struct Literal {
    pub id: usize,
    pub lit_type: LitType,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LitType {
    Natural,
    String,
    Null,
    Syscall,
    Args,
}

impl Info {
    pub const NULL: Self = Self {
        scope_id: 0,
        typ: Typ::Literal(Literal {
            id: 0,
            lit_type: LitType::Null,
        }),
    };

    pub fn typ(typ: Typ) -> Self {
        Self { scope_id: 0, typ }
    }

    pub fn lit_string(id: usize) -> Self {
        Self::typ(Typ::lit(id, LitType::String))
    }

    pub fn lit_natural(id: usize) -> Self {
        Self::typ(Typ::natural(id))
    }

    pub fn syscall(id: usize) -> Self {
        Self::typ(Typ::lit(id, LitType::Syscall))
    }

    pub fn address(id: usize, ret_t: &Self) -> Self {
        Self::typ(Typ::address(id, ret_t))
    }

    pub fn args() -> Self {
        Self::typ(Typ::args())
    }

    pub fn as_local_ref(&self) -> Result<&Local> {
        let Self { typ, .. } = self;
        match typ {
            Typ::Local(a) => Ok(a),
            other => temg!("Not a local variable symbol: {:?}", other),
        }
    }
    pub fn fp_off(&self) -> Result<usize> {
        self.as_local_ref().map(|l| l.fp_off)
    }
    pub fn as_addr_ref(&self) -> Result<&Address> {
        let Self { typ, .. } = self;
        match typ {
            Typ::Address(a) => Ok(a),
            other => temg!("Not an address symbol: {:?}", other),
        }
    }
    pub fn addr(&self) -> Result<usize> {
        self.as_addr_ref().map(|i| i.addr)
    }
    pub fn val(&self) -> usize {
        match self.typ {
            Typ::Address(Address { addr: v, .. })
            | Typ::Local(Local { fp_off: v, .. })
            | Typ::Literal(Literal { id: v, .. }) => v,
        }
    }
    pub fn val_mut(&mut self) -> &mut usize {
        match &mut self.typ {
            Typ::Address(Address { addr: v, .. })
            | Typ::Local(Local { fp_off: v, .. })
            | Typ::Literal(Literal { id: v, .. }) => v,
        }
    }
    pub fn just(val: usize) -> Self {
        let mut si = Self::default();
        *si.val_mut() = val;
        si
    }
}

impl Default for Info {
    fn default() -> Self {
        Self {
            typ: Typ::Local(Local {
                fp_off: 0,
                is_alias: true,
                types: vec![],
            }),
            scope_id: 0,
        }
    }
}

impl Local {
    pub fn foreach<F, R>(&self, mut f: F) -> impl Iterator<Item = R>
    where
        F: FnMut(usize) -> R,
    {
        let &Self { fp_off, .. } = self;
        let size = self.size();
        (0..size).into_iter().map(move |j| {
            let i = fp_off - (size - 1 - j) as usize;
            f(i)
        })
    }

    pub fn size(&self) -> u16 {
        self.types.iter().map(|t| t.typ.size()).sum()
    }
}

impl Typ {
    pub fn size(&self) -> u16 {
        match self {
            Typ::Literal(_) | Typ::Address(_) => 1,
            Typ::Local(local) => local.size(),
        }
    }

    pub fn local<T>(types: T, is_alias: bool, fp_off: usize) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Info>,
    {
        Self::Local(Local {
            fp_off,
            is_alias,
            types: types.into_iter().map(<_>::into).collect(),
        })
    }

    pub fn array<T>(types: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Info>,
    {
        Self::local(types, true, 0)
    }

    pub fn natural(id: usize) -> Self {
        Self::Literal(Literal {
            id,
            lit_type: LitType::Natural,
        })
    }

    pub fn address(addr: usize, ret_t: &Info) -> Self {
        Self::Address(Address {
            addr,
            ret_t: Box::new(ret_t.to_owned()),
        })
    }

    pub fn lit(id: usize, lit_type: LitType) -> Self {
        Self::Literal(Literal { id, lit_type })
    }

    pub fn args() -> Self {
        Self::Literal(Literal {
            id: 0,
            lit_type: LitType::Args,
        })
    }
}

impl fmt::Debug for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { scope_id, typ } = self;
        write!(f, "{:?} @{}", typ, scope_id)
    }
}

impl fmt::Debug for Typ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(local) => local.fmt(f),
            Self::Address(addr) => addr.fmt(f),
            Self::Literal(lit) => lit.fmt(f),
        }
    }
}
impl fmt::Debug for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            fp_off,
            is_alias,
            types,
        } = self;
        let alias = if *is_alias { " alias" } else { "" };
        write!(f, "${}{} :", fp_off, alias)?;
        for typ in types {
            write!(f, "{:?}", typ)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { addr, ret_t } = self;
        write!(f, "*{}: {:?}", addr, ret_t)?;
        Ok(())
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { id, lit_type } = self;
        write!(f, "{:?}({})", lit_type, id)?;
        Ok(())
    }
}
