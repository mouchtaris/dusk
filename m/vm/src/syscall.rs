use {
    super::{mem, value, Job, Result, Value, Vm},
    error::{ldebug, te, temg},
    std::{
        fmt::Write,
        process::{Command, Stdio},
    },
};

pub fn spawn(vm: &mut Vm) -> Result<()> {
    vm.prepare_call();

    let mut sbuf = <_>::default();
    te!(expand_args(vm, &mut sbuf));
    let args: Vec<&str> = sbuf.into_iter().collect();
    let mut sbuf1 = <_>::default();
    te!(expand_envs(vm, &mut sbuf1));
    let envs: Vec<&str> = sbuf1.into_iter().collect();

    let &nargs: &usize = te!(vm.arg_get(0));
    let cwd: &Value = te!(vm.arg_get_val(nargs + 1));
    let target: &Value = te!(vm.arg_get_val(nargs + 2));
    let &inp_redir_n: &usize = te!(vm.arg_get(nargs + 3));
    let &nenvs: &usize = te!(vm.arg_get(nargs + 3 + inp_redir_n + 1));

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
    nenvs       : {nenvs:?}
    envs        : {envs:?}
",
        target = target,
        cwd = cwd,
        nargs = nargs,
        vmargs = vmargs,
        args = args,
        redir = inp_redirs,
        nenvs = nenvs,
        envs = envs,
    );

    let target: &str = te!(vm.val_as_str(&target));

    let mut cmd = Command::new(target);
    cmd.stdin(Stdio::null());

    {
        fn addr(vm: &Vm, i: usize) -> Result<(usize, usize)> {
            let ptr = vm.arg_addr(i);
            let len = 1;
            Ok((te!(ptr), len))
        }
        te!(install_args(vm, &mut cmd, addr, &mut String::new()));
    }

    if let Ok(cwd) = vm.val_as_str(&cwd) {
        cmd.current_dir(cwd);
    }

    let mut job: Job = cmd.into();
    enum Id {
        Job,
        Str,
    }
    let mut inp_jobs: Vec<(Id, usize)> = vec![];

    for redir in inp_redirs {
        inp_jobs.push(match redir {
            &Value::Job(value::Job(jobid)) => (Id::Job, jobid),
            &Value::LitString(value::LitString(strid)) => (Id::Str, strid),
            other => temg!("internal error: {:?}", other),
        })
    }
    for inp_job in inp_jobs {
        let inp_job = match inp_job {
            (Id::Job, jobid) => mem::take(te!(vm.get_job_mut(jobid))),
            (Id::Str, strid) => {
                // TODO this command is filler
                let cmd = Command::new("false");
                let string = te!(vm.get_string_id(strid)).to_owned();
                let buffer = job::Buffer::String(cmd, string);
                Job::Buffer(buffer)
            }
        };
        te!(job.add_input_job(inp_job));
    }

    let job_id = vm.add_job(job);

    vm.allocate(1);
    let val: Value = value::Job(job_id).into();
    ldebug!("put {:?} to {}", val, vm.stackp());
    vm.push_val(val);
    te!(vm.set_ret_val_from_local(0));
    te!(vm.return_from_call(0));
    vm.dealloc(1);

    Ok(())
}

fn expand_arg(vm: &mut Vm, sbuf: &mut buf::StringBuf, arg_addr: usize) -> Result<()> {
    let arg: &Value = vm.stack_get_val(arg_addr);
    match arg {
        &Value::LitString(value::LitString(strid)) => {
            let arg: &str = te!(vm.get_string_id(strid));
            sbuf.add(arg);
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                te!(expand_arg(vm, sbuf, ptr - i));
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
        Value::DynString(value::DynString(string)) => {
            sbuf.add(string);
        }
        other => temg!("Cannot expand arg@{}: {:?}", arg_addr, other),
    }
    Ok(())
}

fn expand_args(vm: &mut Vm, sbuf: &mut buf::StringBuf) -> Result<()> {
    expand_args2(vm, sbuf, |vm, i| Ok(te!(vm.arg_addr(i))))
}

fn expand_args2<G>(vm: &mut Vm, sbuf: &mut buf::StringBuf, argaddr: G) -> Result<()>
where
    G: Fn(&mut Vm, usize) -> Result<usize>,
{
    expand_args3(vm, sbuf, |vm, i| Ok((te!(argaddr(vm, i)), 1)))
}

fn expand_args3<G>(vm: &mut Vm, sbuf: &mut buf::StringBuf, argaddr: G) -> Result<()>
where
    G: Fn(&mut Vm, usize) -> Result<(usize, usize)>,
{
    let (nargs_addr, _) = te!(argaddr(vm, 0));
    let &nargs: &usize = te!(vm.stack_get(nargs_addr));
    ldebug!("expanding {} args from {}", nargs, nargs_addr);

    for i in 1..=nargs {
        let (addr, len) = te!(argaddr(vm, i));
        for j in 0..len {
            te!(expand_arg(vm, sbuf, addr + j));
        }
    }
    Ok(())
}

fn expand_envs(vm: &mut Vm, sbuf: &mut buf::StringBuf) -> Result<()> {
    let &nargs: &usize = te!(vm.arg_get(0));
    let &ninpredr: &usize = te!(vm.arg_get(nargs + 3));
    //let &nenvs: &usize = te!(vm.arg_get(nargs + 3 + ninpredr + 1));

    expand_args3(vm, sbuf, |vm, i| {
        Ok((te!(vm.arg_addr(nargs + 3 + ninpredr + 1 + (i * 2))), 2))
    })
}

type Addr = fn(&Vm, usize) -> Result<(usize, usize)>;

const ADDR_ENV: Addr = |vm, i| {
    let &nargs: &usize = te!(vm.arg_get(0));
    let &ninpredr: &usize = te!(vm.arg_get(nargs + 3));
    Ok((te!(vm.arg_addr(nargs + 3 + ninpredr + 1 + (i * 2))), 2))
};

fn install_args<G>(vm: &Vm, cmd: &mut Command, argaddr: G, sbuf: &mut String) -> Result<()>
where
    G: Fn(&Vm, usize) -> Result<(usize, usize)>,
{
    let (nargs_addr, _) = te!(argaddr(vm, 0));
    let &nargs: &usize = te!(vm.stack_get(nargs_addr));
    ldebug!("injecting {} args from {}", nargs, nargs_addr);

    for i in 1..=nargs {
        let (addr, len) = te!(argaddr(vm, i));
        for j in 0..len {
            te!(inject_arg(vm, cmd, addr + j, sbuf));
        }
    }
    Ok(())
}

fn inject_arg(vm: &Vm, cmd: &mut Command, arg_addr: usize, sbuf: &mut String) -> Result<()> {
    let arg: &Value = vm.stack_get_val(arg_addr);
    match arg {
        &Value::LitString(value::LitString(strid)) => {
            let arg: &str = te!(vm.get_string_id(strid));
            cmd.arg(arg);
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                te!(inject_arg(vm, cmd, ptr - i, sbuf));
            }
        }
        Value::Natural(n) => {
            sbuf.clear();
            te!(write!(sbuf, "{}", n));
            cmd.arg(sbuf);
        }
        &Value::Job(value::Job(jobid)) => {
            let job = te!(vm.get_job(jobid));
            let s = te!(job.as_str());
            cmd.arg(s);
        }
        Value::DynString(value::DynString(string)) => {
            cmd.arg(string);
        }
        other => temg!("Cannot expand arg@{}: {:?}", arg_addr, other),
    }
    Ok(())
}
