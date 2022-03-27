use {
    super::{mem, value, Job, Result, Value, Vm},
    error::{ldebug, te, temg},
};

pub fn spawn(vm: &mut Vm) -> Result<()> {
    vm.prepare_call();

    let &nargs: &usize = te!(vm.arg_get(0));
    let sbuf = te!(expand_args(vm, <_>::default()));
    let args: Vec<&str> = sbuf.seg_vec_in(<_>::default());
    let cwd: &Value = te!(vm.arg_get_val(nargs + 1));
    let target: &Value = te!(vm.arg_get_val(nargs + 2));
    let &inp_redir_n: &usize = te!(vm.arg_get(nargs + 3));

    type RV<'a> = Result<Vec<&'a Value>>;
    let vmargs: RV = (0..=nargs).map(|i| vm.arg_get_val(i)).collect();
    let vmargs = te!(vmargs);
    let inp_redirs: RV = (0..inp_redir_n)
        .map(|i| vm.arg_get_val(nargs + 3 + 1 + i))
        .collect();
    let inp_redirs = te!(inp_redirs);

    ldebug!(
        "[create_job]
    nargs       : {nargs:?}
    target      : {target:?}
    cwd         : {cwd:?}
    vmargs      : {vmargs:?}
    args        : {args:?}
    redir       : {redir:?}
",
        target = target,
        cwd = cwd,
        nargs = nargs,
        vmargs = vmargs,
        args = args,
        redir = inp_redirs,
    );

    use std::process::{Command, Stdio};

    let target: &str = te!(vm.val_as_str(&target));
    let mut cmd = Command::new(target);
    cmd.stdin(Stdio::null());
    cmd.args(&args);
    if let Ok(cwd) = vm.val_as_str(&cwd) {
        cmd.current_dir(cwd);
    }

    let mut job: Job = cmd.into();
    let mut inp_jobs = vec![];

    for redir in inp_redirs {
        match redir {
            &Value::Job(value::Job(jobid)) => inp_jobs.push(jobid),
            other => panic!("{:?}", other),
        }
    }
    for jobid in inp_jobs {
        let inp_job = mem::take(te!(vm.get_job_mut(jobid)));
        te!(job.add_input_job(inp_job));
    }

    let job_id = vm.add_job(job);

    vm.allocate(1);
    let val: Value = value::Job(job_id).into();
    ldebug!("put {:?} to {}", val, vm.stackp());
    vm.push_val(val);
    te!(vm.set_ret_val(0));
    te!(vm.return_from_call(0));
    vm.dealloc(1);

    Ok(())
}

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
        &Value::Job(value::Job(jobid)) => {
            let job = te!(vm.get_job_mut(jobid));
            let s = te!(job.make_string());
            sbuf.add(s);
        }
        other => temg!("Cannot expand arg@{}: {:?}", arg_addr, other),
    }
    Ok(sbuf)
}

fn expand_args(vm: &mut Vm, mut sbuf: buf::StringBuf) -> Result<buf::StringBuf> {
    let &nargs: &usize = te!(vm.arg_get(0));
    ldebug!("expanding {} args from {}", nargs, te!(vm.arg_addr(1)));

    for i in 1..=nargs {
        sbuf = te!(expand_arg(vm, sbuf, te!(vm.arg_addr(i))));
    }
    Ok(sbuf)
}
