use super::Result;

#[derive(Debug, Clone)]
pub struct Info {
    pub typ: Typ,
    pub scope_id: usize,
}

#[derive(Debug, Clone)]
pub enum Typ {
    Local(Local),
    //Address(Address),
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
        }
    }
    pub fn fp_off(&self) -> Result<usize> {
        self.as_local_ref().map(|l| l.fp_off)
    }
    //pub fn as_addr_ref(&self) -> Option<&Address> {
    //    let Self { typ, .. } = self;
    //    match typ {
    //        Typ::Address(a) => Some(a),
    //        _ => None,
    //    }
    //}
}
