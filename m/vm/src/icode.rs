use std::io;

use super::{ldebug, ltrace, soft_todo, te, terr, BorrowMut, Deq, Entry, Map, Result, Vm};

fn _use() {
    soft_todo!();
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct ICode {
    pub instructions: Deq<Instr>,
    pub strings: Map<String, StringInfo>,
}

#[derive(
    Default,
    Debug,
    Copy,
    Eq,
    Ord,
    Hash,
    PartialEq,
    PartialOrd,
    Clone,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct StringInfo {
    pub id: usize,
}

#[derive(
    Debug, Copy, Eq, Ord, Hash, PartialEq, PartialOrd, Clone, serde::Serialize, serde::Deserialize,
)]
pub enum Instr {
    Allocate { size: usize },
    Jump { addr: usize },

    Return,
    Dealloc(usize),
    PushNull,
    PushStr(usize),
    PushNat(usize),
    PushArgs,
    PushLocal(usize),
    Call(usize),
    CleanUp(usize),
    CleanUpCollect(usize),

    SysCall(u8),

    Init,
    SetNatural { value: usize, dst: usize },
    FindInBinPath { id: usize, dst: usize },
    CompleteProcessJob { jobid: usize },
}

impl Instr {
    pub fn operate_on(&self, vm: &mut Vm) -> Result<()> {
        ltrace!("Instr {} {:?}", vm.instr_addr() - 1, self);
        match self {
            Self::Init => {}
            &Self::Allocate { size } => {
                vm.allocate(size);
            }
            &Self::FindInBinPath { id, dst } => {
                let mut id: String = te!(vm.frame_take(id));
                ltrace!("[FindInBinPath] {}", id);

                let len = id.len();
                let mut found = false;
                for bin_path in &vm.bin_path {
                    id.insert(0, '/');
                    id.insert_str(0, bin_path);
                    if let Ok(_) = std::fs::metadata(&id) {
                        found = true;
                        break;
                    }
                    id.replace_range(0..id.len() - len, "");
                }
                if found {
                    ldebug!("[FindInBinPath] found {}", id);
                    vm.frame_set(dst, id);
                } else {
                    te!(Err(format!("Did not find {} in BIN_PATH", id)))
                }
            }
            &Self::CompleteProcessJob { .. } => panic!(),
            &Self::PushNull => vm.push_null(),
            &Self::PushNat(id) => vm.push_val(id),
            &Self::PushStr(id) => vm.push_str(id),
            &Self::SetNatural { value, dst } => vm.frame_set(dst, value),
            &Self::Jump { addr } => vm.jump(addr),
            &Self::SysCall(id) => te!(crate::syscall::call(vm, id)),
            &Self::Return => vm.return_from_call(),
            &Self::Dealloc(size) => vm.dealloc(size),
            &Self::PushArgs => te!(vm.push_args()),
            &Self::PushLocal(fp_off) => vm.push_local(fp_off),
            &Self::Call(addr) => {
                vm.prepare_call();
                vm.jump(addr);
            }
            &Self::CleanUp(fp_off) => te!(vm.cleanup(fp_off)),
            &Self::CleanUpCollect(fp_off) => te!(vm.cleanup_collect(fp_off)),
        }
        Ok(())
    }

    pub fn allocate_size(&mut self, size: usize) -> Result<()> {
        let me = self;
        Ok(*match me {
            Self::Allocate { size } => size,
            _ => terr!("not an allocate instr"),
        } = size)
    }
}

impl StringInfo {
    pub fn add<S, C>(mut icode: C, s: S) -> Result<StringInfo>
    where
        S: Into<String>,
        C: BorrowMut<ICode>,
    {
        let t = &mut icode.borrow_mut().strings;
        let id = t.len();
        match t.entry(s.into()) {
            Entry::Occupied(occ) => Ok(occ.get().clone()),
            Entry::Vacant(vac) => Ok(vac.insert(StringInfo { id }).to_owned()),
        }
    }
}
impl ICode {
    pub fn write_to<O>(&self, out: io::Result<O>) -> io::Result<()>
    where
        O: io::Write,
    {
        let ilen = usize::to_le_bytes(self.instructions.len());
        let slen = usize::to_le_bytes(self.strings.len());
        out.and_then(|mut out| {
            out.write_all(&slen)?;
            for (sval, info) in &self.strings {
                let s = sval.as_bytes();
                let slen = usize::to_le_bytes(s.len());
                out.write_all(&slen)?;
                out.write_all(s)?;
                let strid = usize::to_le_bytes(info.id);
                out.write_all(&strid)?;
            }
            out.write_all(&ilen)?;
            for instr in &self.instructions {
                let (code, arg0) = match *instr {
                    Instr::Allocate { size } => (0x00, size),
                    Instr::Jump { addr } => (0x01, addr),
                    Instr::Return => (0x02, 0x00),
                    Instr::PushNull => (0x03, 0x00),
                    Instr::PushStr(strid) => (0x04, strid),
                    Instr::PushNat(val) => (0x05, val),
                    Instr::SysCall(id) => (0x06, id as usize),
                    Instr::Dealloc(size) => (0x07, size),
                    Instr::PushArgs => (0x08, 0x00),
                    Instr::PushLocal(fp_off) => (0x09, fp_off),
                    Instr::Call(addr) => (0x0a, addr),
                    Instr::CleanUp(fp_off) => (0x0b, fp_off),
                    Instr::CleanUpCollect(fp_off) => (0x0c, fp_off),
                    _ => panic!(),
                };
                let code = usize::to_le_bytes(code);
                let arg = usize::to_le_bytes(arg0);
                out.write_all(&code)?;
                out.write_all(&arg)?;
            }
            Ok(())
        })
    }
    pub fn load_from<I>(inp: io::Result<I>) -> Result<Self>
    where
        I: io::Read,
    {
        let mut usize_buf = usize::to_le_bytes(0usize);
        let mut icode = ICode::default();
        let mut byte_buf = Vec::new();

        let inp = Ok(te!(inp));
        inp.and_then(|mut inp| {
            te!(inp.read_exact(&mut usize_buf));
            let slen = usize::from_le_bytes(usize_buf);
            icode.strings.reserve(slen);
            for _ in 0..slen {
                te!(inp.read_exact(&mut usize_buf));
                let slen = usize::from_le_bytes(usize_buf);

                byte_buf.resize(slen, 0x00);
                te!(inp.read_exact(&mut byte_buf));

                te!(inp.read_exact(&mut usize_buf));
                let strid = usize::from_le_bytes(usize_buf);

                let s = te!(String::from_utf8(byte_buf.clone()));
                let info = StringInfo { id: strid };
                icode.strings.insert(s, info);
            }

            te!(inp.read_exact(&mut usize_buf));
            let ilen = usize::from_le_bytes(usize_buf);
            icode.instructions.reserve(ilen);
            for _ in 0..ilen {
                te!(inp.read_exact(&mut usize_buf));
                let code = usize::from_le_bytes(usize_buf);
                te!(inp.read_exact(&mut usize_buf));
                let val = usize::from_le_bytes(usize_buf);
                let instr = match code {
                    0x00 => Instr::Allocate { size: val },
                    0x01 => Instr::Jump { addr: val },
                    0x02 => Instr::Return,
                    0x03 => Instr::PushNull,
                    0x04 => Instr::PushStr(val),
                    0x05 => Instr::PushNat(val),
                    0x06 => Instr::SysCall(val as u8),
                    0x07 => Instr::Dealloc(val),
                    0x08 => Instr::PushArgs,
                    0x09 => Instr::PushLocal(val),
                    0x0a => Instr::Call(val),
                    0x0b => Instr::CleanUp(val),
                    0x0c => Instr::CleanUpCollect(val),
                    _ => panic!(),
                };
                icode.instructions.push_back(instr);
            }
            Ok(icode)
        })
    }
}
