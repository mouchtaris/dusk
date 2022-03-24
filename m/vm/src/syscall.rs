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
        &Value::LitString(value::LitString(strid)) => {
            let arg: &str = te!(vm.get_string_id(strid));
            sbuf.add(arg);
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                sbuf = te!(expand_arg(vm, sbuf, ptr - i));
            }
        }
        Value::Natural(n) => {
            sbuf.add(n);
        }
        other => temg!("Cannot expand arg@{}: {:?}", arg_addr, other),
    }
    Ok(sbuf)
}

fn expand_args(vm: &mut Vm, mut sbuf: buf::StringBuf) -> Result<buf::StringBuf> {
    let &nargs: &usize = te!(vm.arg_get(0));
    let nargs = nargs; // nargs, target, cwd

    for i in 1..=nargs {
        sbuf = te!(expand_arg(vm, sbuf, te!(vm.arg_addr(i))));
    }
    Ok(sbuf)
}

mod handlers {
    use super::*;
    pub mod create_job {
        use super::*;
        pub const H: Handler = |vm| {
            let &nargs: &usize = te!(vm.arg_get(0));
            let cwd: Value = te!(vm.arg_take_val(nargs + 1));
            let target: Value = te!(vm.arg_take_val(nargs + 2));
            let sbuf = te!(expand_args(vm, <_>::default()));
            let args: Vec<&str> = sbuf.seg_vec_in(<_>::default());
            let vmargs: Result<Vec<&Value>> = (0..=nargs).map(|i| vm.arg_get_val(i)).collect();
            let vmargs = te!(vmargs);
            ldebug!(
                "[create_job]
    nargs       : {nargs:?}
    target      : {target:?}
    cwd         : {cwd:?}
    vmargs      : {vmargs:?}
    args        : {args:?}
",
                target = target,
                cwd = cwd,
                nargs = nargs,
                vmargs = vmargs,
                args = args,
            );

            use std::process::Command;

            let target = te!(vm.val_as_str(&target));

            let mut cmd = Command::new(target);

            cmd.args(&args);
            if let Ok(cwd) = vm.val_as_str(&cwd) {
                cmd.current_dir(cwd);
            }

            let child = te!(cmd.spawn());
            let proc_id = vm.add_process(child);

            let retval: &mut Value = te!(vm.arg_get_val_mut(nargs + 3));
            *retval = value::Job(proc_id).into();

            vm.return_from_call();

            Ok(())
        };
    }
}
