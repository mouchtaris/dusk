pub const CREATE_JOB: u8 = 0x10;

use {
    super::{Result, Vm},
    error::{ldebug, te, temg},
};

pub fn call(vm: &mut Vm, id: u8) -> Result<()> {
    vm.prepare_call();
    match id {
        CREATE_JOB => handlers::create_job::H(vm),
        other => panic!("Invalid syscall id: {}", other),
    }
}

type Handler = fn(&mut Vm) -> Result<()>;

mod handlers {
    use super::*;
    pub mod create_job {
        use super::*;
        pub const H: Handler = |vm| {
            let job_type: &usize = te!(vm.arg_get(0));
            let target = vm.arg_get_val(1);
            let cwd = vm.arg_get_val(2);
            let &nargs: &usize = te!(vm.arg_get(3));
            let mut args = Vec::<&String>::new();
            for i in 0..nargs {
                args.push(te!(vm.arg_get(4 + i)));
            }
            ldebug!(
                "[create_job]
    job_type    : {job_type:?}
    target      : {target:?}
    cwd         : {cwd:?}
    nargs       : {nargs:?}
    args        : {args:?}
",
                target = target,
                job_type = job_type,
                cwd = cwd,
                nargs = nargs,
                args = args,
            );

            match job_type {
                0 => {
                    use std::process::{Child, Command, ExitStatus};

                    let target: &String = te!(target.try_ref());
                    let mut cmd: Command = Command::new(target);

                    cmd.args(&args);

                    if let Ok(cwd) = cwd.try_ref::<String>() {
                        cmd.current_dir(cwd);
                    }

                    let mut child: Child = te!(cmd.spawn());

                    let status: ExitStatus = te!(child.wait());

                    if !status.success() {
                        temg!("Subprocess failed: {:?}", status)
                    }

                    vm.return_from_call();
                }
                1 => {
                    let &target: &usize = te!(target.try_ref());
                    vm.jump(target);
                }
                _ => panic!(),
            }

            Ok(())
        };
    }
}
