use {
    super::{ldebug, ltrace, soft_todo, te, value, BorrowMut, Deq, Entry, Map, Result, Vm},
    value::ProcessBuilder,
};

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
    Init,
    Allocate { size: usize },
    LoadString { strid: usize, dst: usize },
    FindInBinPath { id: usize, dst: usize },
    CreateProcessJob { path: usize, dst: usize },
    CompleteProcessJob,
    JobSetCwd,
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
            Self::CompleteProcessJob => {
                soft_todo!();
            }
            Self::JobSetCwd => {
                let _proc = te!(vm.rv_mut::<ProcessBuilder>());
                todo!()
            }
            &Self::LoadString { strid, dst } => {
                vm.load_string(strid, dst);
            }
        }
        Ok(vm)
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
            Entry::Occupied(occ) => te!(Err(format!("Re-insert string: {}", occ.key()))),
            Entry::Vacant(vac) => Ok(vac.insert(StringInfo { id }).to_owned()),
        }
    }
}
