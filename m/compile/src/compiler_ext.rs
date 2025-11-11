use super::{Borrow, BorrowMut, Compiler, HashMap};

impl<S: Borrow<Compiler>> CompilerExt for S {}
pub trait CompilerExt: Borrow<Compiler> {
    fn strings(&self) -> HashMap<usize, &str> {
        let Compiler { icode, .. } = self.borrow();
        icode
            .strings
            .iter()
            .map(|(k, &vm::StringInfo { id })| (id, k.as_str()))
            .collect()
    }

    fn get_string_id(&self, s: impl AsRef<str>) -> Option<usize> {
        let Compiler { icode, .. } = self.borrow();
        icode
            .strings
            .get(s.as_ref())
            .map(|&vm::StringInfo { id, .. }| id)
    }

    fn num_instrs(&self) -> usize {
        let Compiler { icode, .. } = self.borrow();
        icode.instructions.len()
    }
}

impl<S: BorrowMut<Compiler>> CompilerMut for S {}
pub trait CompilerMut: BorrowMut<Compiler> {
    fn add_string(&mut self, s: impl Into<String>) -> usize {
        let Compiler { icode, .. } = self.borrow_mut();
        icode.add_string(s)
    }
}

impl<S: BorrowMut<vm::ICode>> VmICodeMut for S {}
pub trait VmICodeMut: BorrowMut<vm::ICode> {
    fn add_string(&mut self, s: impl Into<String>) -> usize {
        add_string(&mut icode_mut(self).strings, s)
    }
    fn emit<I: Into<vm::Instr>>(&mut self, instrs: impl IntoIterator<Item = I>) -> usize {
        let vm::ICode { instructions, .. } = icode_mut(self);
        instrs_emit(instructions, instrs)
    }
}
impl<S: Borrow<vm::ICode>> VmICodeRef for S {}
pub trait VmICodeRef: Borrow<vm::ICode> {
    fn get_string_id(&self, s: impl AsRef<str>) -> Option<usize> {
        get_string_id(&icode(self).strings, s)
    }
}
fn icode_mut(this: &mut (impl ?Sized + BorrowMut<vm::ICode>)) -> &mut vm::ICode {
    this.borrow_mut()
}
fn icode(this: &(impl ?Sized + Borrow<vm::ICode>)) -> &vm::ICode {
    this.borrow()
}
pub fn get_string_id(this: &vm::Strings, s: impl AsRef<str>) -> Option<usize> {
    this.get(s.as_ref()).map(|&vm::StringInfo { id, .. }| id)
}
pub fn add_string(this: &mut vm::Strings, s: impl Into<String>) -> usize {
    vm::StringInfo::add_to_strings(this, s).id
}
pub fn instrs_emit<I: Into<vm::Instr>>(
    this: &mut vm::Instrs,
    instrs: impl IntoIterator<Item = I>,
) -> usize {
    let instructions = this;

    let a = instructions.len();

    instructions.extend(instrs.into_iter().map(<_>::into));

    instructions.len() - a
}
