use super::{temg, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Info {
    pub typ: Typ,
    pub scope_id: usize,
}

either::either![
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub Typ,
        Local,
        Address
];
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Local {
    pub fp_off: usize,
    pub is_alias: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Address {
    pub addr: usize,
}

impl Info {
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
            Typ::Address(Address { addr: v }) => v,
            Typ::Local(Local { fp_off: v, .. }) => v,
        }
    }
    pub fn just(val: usize) -> Self {
        Self {
            typ: Typ::Local(Local {
                fp_off: val,
                is_alias: true,
            }),
            scope_id: 0,
        }
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
