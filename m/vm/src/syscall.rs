pub const CREATE_JOB: u8 = 0x10;

use {
    super::{value, Result, Value, Vm},
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

fn expand_arg(vm: &mut Vm, mut sbuf: buf::StringBuf, arg_addr: usize) -> Result<buf::StringBuf> {
    let arg: &Value = vm.stack_get_val(arg_addr);
    match arg {
        Value::String(arg) => {
            sbuf.add_str(arg);
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                sbuf = te!(expand_arg(vm, sbuf, ptr - i));
            }
        }
        other => temg!("Cannot expand arg@{}: {:?}", arg_addr, other),
    }
    Ok(sbuf)
}

fn expand_args(vm: &mut Vm, mut sbuf: buf::StringBuf) -> Result<buf::StringBuf> {
    let &nargs: &usize = te!(vm.arg_get(3));

    for i in 0..nargs {
        sbuf = te!(expand_arg(vm, sbuf, vm.arg_addr(4 + i)));
    }
    Ok(sbuf)
}

mod handlers {
    use super::*;
    pub mod create_job {
        use super::*;
        pub const H: Handler = |vm| {
            let &job_type: &usize = te!(vm.arg_get(0));
            let target: Value = vm.arg_take_val(1);
            let cwd: Value = vm.arg_take_val(2);
            let &nargs: &usize = te!(vm.arg_get(3));
            ldebug!(
                "[create_job]
    job_type    : {job_type:?}
    target      : {target:?}
    cwd         : {cwd:?}
    nargs       : {nargs:?}
",
                target = target,
                job_type = job_type,
                cwd = cwd,
                nargs = nargs,
            );

            let sbuf = te!(expand_args(vm, <_>::default()));
            let args = sbuf.seg_vec_in(<_>::default());
            ldebug!("args: {args:?}", args = args);

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
