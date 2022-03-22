use super::{temg, Result};

#[derive(Debug, Clone)]
pub struct Info {
    pub typ: Typ,
    pub scope_id: usize,
}

#[derive(Debug, Clone)]
pub enum Typ {
    Local(Local),
    Address(Address),
}
#[derive(Debug, Clone)]
pub struct Local {
    pub fp_off: usize,
}
#[derive(Debug, Clone)]
pub struct Address {
    pub addr: usize,
}

impl Info {
    pub fn as_local_ref(&self) -> Result<&Local> {
        let Self { typ, .. } = self;
        match typ {
            Typ::Local(a) => Ok(a),
            _ => temg!("Not a local symbol"),
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
            Typ::Local(Local { fp_off: v }) => v,
        }
    }
    pub fn just(val: usize) -> Self {
        Self {
            typ: Typ::Local(Local { fp_off: val }),
            scope_id: 0,
        }
    }
}
