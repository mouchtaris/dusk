use std::io;

use super::{
    ltrace, soft_todo, syscall, te, terr, value, BorrowMut, Deq, Entry, Job, Map, Result, Vm,
};

fn _use() {
    soft_todo!();
}

pub type Instrs = Deq<Instr>;
pub type Strings = Map<String, StringInfo>;

#[derive(Default, Debug)]
pub struct ICode {
    pub instructions: Instrs,
    pub strings: Strings,
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
}

impl Instr {
    pub fn operate_on(&self, vm: &mut Vm) -> Result<()> {
        ltrace!("Instr {} {:?}", vm.instr_addr() - 1, self);
        match self {
            &Self::Allocate { size } => {
                vm.allocate(size);
            }
            &Self::PushNull => te!(vm.push_null()),
            &Self::PushNat(id) => te!(vm.push_val(id)),
            &Self::PushStr(id) => te!(vm.push_lit_str(id)),
            &Self::Jump { addr } => vm.jump(addr),
            &Self::Syscall(syscall::SPAWN) => te!(syscall::spawn(vm)),
            &Self::Syscall(syscall::ARG_SLICE) => te!(syscall::argslice(vm)),
            &Self::Syscall(_) => te!(syscall::builtin(vm)),
            &Self::Return(frame_size) => te!(vm.return_from_call(frame_size)),
            &Self::RetLocal(fp_off) => te!(vm.set_ret_val_from_local(fp_off)),
            &Self::PushArgs => te!(vm.push_args()),
            &Self::PushLocal(fp_off) => te!(vm.push_local(fp_off)),
            &Self::PushFuncAddr(addr) => te!(vm.push_val(value::FuncAddr(addr))),
            &Self::PushSysCall(id) => te!(vm.push_val(value::SysCallId(id))),
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
    pub fn write_to<O>(&self, out: io::Result<O>) -> io::Result<()>
    where
        O: io::Write,
    {
        let ilen = usize::to_le_bytes(self.instructions.len());
        out.and_then(|mut out| {
            buf::sd2::WriteOut::write_out(&self.strings, &mut out)?;
            out.write_all(&ilen)?;
            for instr in &self.instructions {
                let (code, arg0) = match *instr {
                    Instr::Allocate { size } => (0x00, size),
                    Instr::Jump { addr } => (0x01, addr),
                    Instr::Return(sp_off) => (0x02, sp_off),
                    Instr::PushNull => (0x03, 0x00),
                    Instr::PushStr(strid) => (0x04, strid),
                    Instr::PushNat(val) => (0x05, val),
                    Instr::Syscall(id) => (0x06, id),
                    Instr::RetLocal(src_fp_off) => (0x07, src_fp_off),
                    Instr::PushArgs => (0x08, 0x00),
                    Instr::PushLocal(fp_off) => (0x09, fp_off),
                    Instr::Call(addr) => (0x0a, addr),
                    Instr::CleanUp(fp_off) => (0x0b, fp_off),
                    Instr::Collect(fp_off) => (0x0c, fp_off),
                    Instr::PushFuncAddr(addr) => (0x0d, addr),
                    Instr::Pipe(addr) => (0x0e, addr),
                    Instr::RetStr(id) => (0x0f, id),
                    Instr::RetNat(val) => (0x10, val),
                    Instr::RetFuncAddr(addr) => (0x11, addr),
                    Instr::PushSysCall(id) => (0x12, id),
                    Instr::BufferString(fp_off) => (0x13, fp_off),
                };
                let code = u8::to_le_bytes(code);
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
        //let mut byte_buf = Vec::new();

        let inp = Ok(te!(inp));
        inp.and_then(|mut inp| {
            icode.strings = te!(buf::sd2::ReadIn::read_in(&mut inp));

            te!(inp.read_exact(&mut usize_buf));
            let ilen = usize::from_le_bytes(usize_buf);
            icode.instructions.reserve(ilen);
            for _ in 0..ilen {
                te!(inp.read_exact(&mut usize_buf[0..1]));
                let code = u8::from_le_bytes([usize_buf[0]]);
                te!(inp.read_exact(&mut usize_buf));
                let val = usize::from_le_bytes(usize_buf);
                let instr = match code {
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
                    other => panic!("{:?}", other),
                };
                icode.instructions.push_back(instr);
            }
            Ok(icode)
        })
    }
}
