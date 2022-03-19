use {
    super::{
        ldebug, ltrace, soft_todo, te, temg, terr, value, BorrowMut, Deq, Entry, Map, Result,
        Value, Vm,
    },
    value::ProcessBuilder,
};

fn _use() {
    soft_todo!();
}

#[derive(Default, Debug)]
pub struct ICode {
    pub instructions: Deq<Instr>,
    pub strings: Map<String, StringInfo>,
}

#[derive(Default, Debug, Copy, Eq, Ord, Hash, PartialEq, PartialOrd, Clone)]
pub struct StringInfo {
    pub id: usize,
}

#[derive(Debug, Copy, Eq, Ord, Hash, PartialEq, PartialOrd, Clone)]
pub enum Instr {
    Allocate { size: usize },
    Jump { addr: usize },

    PushNull,
    PushStr(usize),
    PushNat(usize),

    SysCall(u8),

    Init,
    SetNatural { value: usize, dst: usize },
    FindInBinPath { id: usize, dst: usize },
    CreateProcessJob { path: usize, dst: usize },
    CompleteProcessJob { jobid: usize },
    JobSetCwd { jobid: usize, cwdid: usize },
    JobPushArg { jobid: usize, argid: usize },
}

impl Instr {
    pub fn operate_on(&self, mut vm: Vm) -> Result<Vm> {
        ltrace!("Instr {:?}", self);
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
            &Self::CreateProcessJob { path, dst } => {
                let path: String = te!(vm.frame_take(path));
                let proc = ProcessBuilder::new(path);
                vm.frame_set(dst, proc);
            }
            &Self::CompleteProcessJob { jobid } => match te!(vm.frame_take_value(jobid)) {
                Value::ProcessBuilder(mut cmd) => {
                    ltrace!("Spawning child");
                    use std::process::{Child, ExitStatus};
                    let mut child: Child = te!(cmd.spawn());
                    let status: ExitStatus = te!(child.wait());
                    if !status.success() {
                        temg!("Subprocess failed: {:?}", status)
                    }
                }
                other => temg!("{:?}", other),
            },
            &Self::JobSetCwd { jobid, cwdid } => {
                let cwd: String = te!(vm.frame_take(cwdid));
                let proc: &mut ProcessBuilder = te!(vm.frame_mut(jobid));
                proc.current_dir(cwd);
            }
            &Self::JobPushArg { jobid, argid } => {
                let cwd: String = te!(vm.frame_take(argid));
                let proc: &mut ProcessBuilder = te!(vm.frame_mut(jobid));
                proc.arg(cwd);
            }
            &Self::PushNull => vm.push_null(),
            &Self::PushNat(id) => vm.push_val(id),
            &Self::PushStr(id) => vm.push_str(id),
            &Self::SetNatural { value, dst } => vm.frame_set(dst, value),
            &Self::Jump { addr } => vm.jump(addr),
            &Self::SysCall(id) => te!(crate::syscall::call(&mut vm, id)),
        }
        Ok(vm)
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
