use super::{temg, Result};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Info {
    pub typ: Typ,
    pub scope_id: usize,
}

either::either![
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub Typ,
        Local,
        Address,
        Literal
];
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Local {
    pub fp_off: usize,
    pub is_alias: bool,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Address {
    pub addr: usize,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Literal {
    pub id: usize,
    pub lit_type: LitType,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LitType {
    Natural,
    String,
    Null,
}

impl Info {
    pub const NULL: Self = Self {
        scope_id: 0,
        typ: Typ::Literal(Literal {
            id: 0,
            lit_type: LitType::Null,
        }),
    };

    pub fn lit_string(id: usize) -> Self {
        Self {
            scope_id: 0,
            typ: Typ::Literal(Literal {
                id,
                lit_type: LitType::String,
            }),
        }
    }

    pub fn lit_natural(id: usize) -> Self {
        Self {
            scope_id: 0,
            typ: Typ::Literal(Literal {
                id,
                lit_type: LitType::Natural,
            }),
        }
    }

    pub fn as_local_ref(&self) -> Result<&Local> {
        let Self { typ, .. } = self;
        match typ {
            Typ::Local(a) => Ok(a),
            _ => temg!("Not a local variable symbol"),
        }
    }
    pub fn fp_off(&self) -> Result<usize> {
        self.as_local_ref().map(|l| l.fp_off)
    }
    pub fn as_addr_ref(&self) -> Result<&Address> {
        let Self { typ, .. } = self;
        match typ {
            Typ::Address(a) => Ok(a),
            _ => temg!("Not an address symbol"),
        }
    }
    pub fn addr(&self) -> Result<usize> {
        self.as_addr_ref().map(|i| i.addr)
    }
    pub fn val(&self) -> usize {
        match self.typ {
            Typ::Address(Address { addr: v })
            | Typ::Local(Local { fp_off: v, .. })
            | Typ::Literal(Literal { id: v, .. }) => v,
        }
    }
    pub fn val_mut(&mut self) -> &mut usize {
        match &mut self.typ {
            Typ::Address(Address { addr: v })
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
            }),
            scope_id: 0,
        }
    }
}
