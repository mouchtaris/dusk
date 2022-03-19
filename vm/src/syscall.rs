pub const CREATE_PROCESS_JOB: u8 = 0x10;

use {
    super::{Result, Vm},
    error::{soft_todo, te},
};

pub fn call(vm: &mut Vm, id: u8) -> Result<()> {
    match id {
        CREATE_PROCESS_JOB => handlers::create_process_job::H(vm),
        other => panic!("Invalid syscall id: {}", other),
    }
}

type Handler = fn(&mut Vm) -> Result<()>;

mod handlers {
    use super::*;
    pub mod create_process_job {
        use super::*;
        pub const H: Handler = |vm| {
            let cwd: &String = te!(vm.frame(0));
            soft_todo!();
            Ok(())
        };
    }
}
