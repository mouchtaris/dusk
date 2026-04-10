use std::io;

use super::{ltrace, soft_todo, syscall, te, terr, value, Deq, Entry, Job, Map, Result, Vm};

fn _use() {
    soft_todo!();
}

pub type Instrs = Deq<Instr>;
pub type Strings = Map<String, StringInfo>;
pub type BytesTable = Vec<Vec<u8>>;

#[derive(Default, Debug, Clone)]
pub struct ICode {
    pub instructions: Instrs,
    pub strings: Strings,
    pub bytes: BytesTable,
}

#[derive(Default, Debug, Copy, Eq, Ord, Hash, PartialEq, PartialOrd, Clone)]
pub struct StringInfo {
    pub id: usize,
}
buf::sd_struct![StringInfo, id];

#[derive(Debug, Copy, Eq, Ord, Hash, PartialEq, PartialOrd, Clone)]
pub enum Instr {
    Allocate { size: usize },
    Jump { addr: usize },

    PushNull,
    PushStr(usize),
    PushNat(usize),
    PushFuncAddr(usize),
    PushLocal(usize),
    PushSysCall(usize),
    PushArgs,

    RetLocal(usize),
    RetStr(usize),
    RetNat(usize),
    RetFuncAddr(usize),

    Call(usize),
    Syscall(usize),
    Return(usize),

    CleanUp(usize),
    Collect(usize),
    Pipe(usize),
    BufferString(usize),
    PushBytes(usize),
    RetBytes(usize),
}

impl Instr {
    pub fn operate_on(&self, vm: &mut Vm) -> Result<()> {
        ltrace!("Instr {} {:?}", vm.instr_addr() - 1, self);
        match self {
            &Self::Allocate { size } => {
                vm.allocate(size);
            }
            &Self::PushNull => {
                te!(vm.push_null());
            }
            &Self::PushNat(id) => {
                te!(vm.push_val(id));
            }
            &Self::PushStr(id) => {
                te!(vm.push_lit_str(id));
            }
            &Self::Jump { addr } => vm.jump(addr),
            &Self::Syscall(syscall::SPAWN) => te!(syscall::spawn(vm)),
            &Self::Syscall(syscall::ARG_SLICE) => te!(syscall::argslice(vm)),
            &Self::Syscall(_) => te!(syscall::builtin(vm)),
            &Self::Return(frame_size) => te!(vm.return_from_call(frame_size)),
            &Self::RetLocal(fp_off) => te!(vm.set_ret_val_from_local(fp_off)),
            &Self::PushArgs => {
                te!(vm.push_args());
            }
            &Self::PushLocal(fp_off) => {
                te!(vm.push_local(fp_off));
            }
            &Self::PushFuncAddr(addr) => {
                te!(vm.push_val(value::FuncAddr(addr)));
            }
            &Self::PushSysCall(id) => {
                te!(vm.push_val(value::SysCallId(id)));
            }
            &Self::Call(_) => {
                te!(vm.prepare_call());
                let addr = te!(vm.call_target_func_addr());
                vm.jump(addr);
            }
            &Self::CleanUp(fp_off) => te!(vm.cleanup(fp_off, "", Job::cleanup)),
            &Self::Collect(fp_off) => te!(vm.cleanup(fp_off, "collect", Job::collect)),
            &Self::Pipe(fp_off) => te!(vm.cleanup(fp_off, "pipe", Job::pipe)),
            &Self::BufferString(fp_off) => {
                log::debug!("Collecting {:?}", te!(vm.frame_get_val(fp_off)));
                te!(vm.cleanup(fp_off, "string_buffer", |j| -> Result<()> {
                    te!(Job::make_string(j));
                    Ok(())
                }))
            }
            &Self::RetStr(id) => te!(vm.set_ret_val(value::LitString(id))),
            &Self::RetNat(val) => te!(vm.set_ret_val(val)),
            &Self::RetFuncAddr(addr) => te!(vm.set_ret_val(value::FuncAddr(addr))),
            &Self::PushBytes(id) => {
                te!(vm.push_lit_bytes(id));
            }
            &Self::RetBytes(id) => te!(vm.set_ret_val(value::LitBytes(id))),
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

impl buf::sd2::WriteOut for Instr {
    fn write_out<O: io::Write>(&self, dst: &mut O) -> io::Result<()> {
        let (code, arg0) = match *self {
            Instr::Allocate { size } => (0x00, size),
            Instr::Jump { addr } => (0x01, addr),
            Instr::Return(v) => (0x02, v),
            Instr::PushNull => (0x03, 0x00),
            Instr::PushStr(v) => (0x04, v),
            Instr::PushNat(v) => (0x05, v),
            Instr::Syscall(v) => (0x06, v),
            Instr::RetLocal(v) => (0x07, v),
            Instr::PushArgs => (0x08, 0x00),
            Instr::PushLocal(v) => (0x09, v),
            Instr::Call(v) => (0x0a, v),
            Instr::CleanUp(v) => (0x0b, v),
            Instr::Collect(v) => (0x0c, v),
            Instr::PushFuncAddr(v) => (0x0d, v),
            Instr::Pipe(v) => (0x0e, v),
            Instr::RetStr(v) => (0x0f, v),
            Instr::RetNat(v) => (0x10, v),
            Instr::RetFuncAddr(v) => (0x11, v),
            Instr::PushSysCall(v) => (0x12, v),
            Instr::BufferString(v) => (0x13, v),
            Instr::PushBytes(v) => (0x14, v),
            Instr::RetBytes(v) => (0x15, v),
        };
        dst.write_all(&[code as u8])?;
        dst.write_all(&usize::to_le_bytes(arg0))
    }
}

impl buf::sd2::ReadIn for Instr {
    fn read_in<I: io::Read>(inp: &mut I) -> buf::sd2::Result<Self> {
        let mut code_buf = [0u8; 1];
        let mut val_buf = usize::to_le_bytes(0);
        te!(inp.read_exact(&mut code_buf));
        te!(inp.read_exact(&mut val_buf));
        let val = usize::from_le_bytes(val_buf);
        Ok(match code_buf[0] {
            0x00 => Instr::Allocate { size: val },
            0x01 => Instr::Jump { addr: val },
            0x02 => Instr::Return(val),
            0x03 => Instr::PushNull,
            0x04 => Instr::PushStr(val),
            0x05 => Instr::PushNat(val),
            0x06 => Instr::Syscall(val),
            0x07 => Instr::RetLocal(val),
            0x08 => Instr::PushArgs,
            0x09 => Instr::PushLocal(val),
            0x0a => Instr::Call(val),
            0x0b => Instr::CleanUp(val),
            0x0c => Instr::Collect(val),
            0x0d => Instr::PushFuncAddr(val),
            0x0e => Instr::Pipe(val),
            0x0f => Instr::RetStr(val),
            0x10 => Instr::RetNat(val),
            0x11 => Instr::RetFuncAddr(val),
            0x12 => Instr::PushSysCall(val),
            0x13 => Instr::BufferString(val),
            0x14 => Instr::PushBytes(val),
            0x15 => Instr::RetBytes(val),
            other => panic!("Unknown instruction opcode: {:#x}", other),
        })
    }
}

impl StringInfo {
    pub fn add_to_strings<S>(this: &mut Strings, s: S) -> StringInfo
    where
        S: Into<String>,
    {
        let t = this;
        let id = t.len();
        match t.entry(s.into()) {
            Entry::Occupied(occ) => occ.get().clone(),
            Entry::Vacant(vac) => vac.insert(StringInfo { id }).to_owned(),
        }
    }
}
impl ICode {
    pub fn add_bytes(&mut self, data: Vec<u8>) -> usize {
        let id = self.bytes.len();
        self.bytes.push(data);
        id
    }

    pub fn write_to<O>(&self, out: io::Result<O>) -> io::Result<()>
    where
        O: io::Write,
    {
        out.and_then(|mut out| {
            buf::sd2::WriteOut::write_out(&self.strings, &mut out)?;
            buf::sd2::WriteOut::write_out(&self.bytes, &mut out)?;
            buf::sd2::WriteOut::write_out(&self.instructions, &mut out)?;
            Ok(())
        })
    }
    pub fn load_from<I>(inp: io::Result<I>) -> Result<Self>
    where
        I: io::Read,
    {
        let inp = Ok(te!(inp));
        inp.and_then(|mut inp| {
            let strings = te!(buf::sd2::ReadIn::read_in(&mut inp));
            let bytes = te!(buf::sd2::ReadIn::read_in(&mut inp));
            let instructions = te!(buf::sd2::ReadIn::read_in(&mut inp));
            Ok(ICode {
                strings,
                bytes,
                instructions,
            })
        })
    }
}
