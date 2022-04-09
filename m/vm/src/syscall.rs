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

    let &nargs: &usize = te!(vm.arg_get(0));
    let cwd: &Value = te!(vm.arg_get_val(nargs + 1));
    let target: &Value = te!(vm.arg_get_val(nargs + 2));
    let &inp_redir_n: &usize = te!(vm.arg_get(nargs + 3));
    let &nenvs: &usize = te!(vm.arg_get(nargs + 3 + inp_redir_n + 1));

    type RV<'a> = Result<Vec<&'a Value>>;

    let vmargs: RV = (0..=nargs).map(|i| vm.arg_get_val(i)).collect();
    let vmargs = te!(vmargs);

    ldebug!(
        "[create_job]
    nargs       : {nargs:?}
    target      : {target:?}
    cwd         : {cwd:?}
    vmargs      : {vmargs:?}
    nenvs       : {nenvs:?}
",
        target = target,
        cwd = cwd,
        nargs = nargs,
        vmargs = vmargs,
        nenvs = nenvs,
    );

    let target: &str = te!(vm.val_as_str(&target));

    let mut cmd = Command::new(target);
    cmd.stdin(Stdio::null());

    if let Ok(cwd) = vm.val_as_str(&cwd) {
        cmd.current_dir(cwd);
    }

    te!(install_args(
        vm,
        &mut |arg| {
            cmd.arg(arg);
        },
        ADDR_ARG,
        &mut String::new()
    ));
    {
        let mut key = String::new();
        te!(install_args(
            vm,
            &mut |env| {
                if key.is_empty() {
                    key.push_str(env);
                } else {
                    eprintln!("Set ENV {} = {}", key, env);
                    cmd.env(key.as_str(), env);
                    key.clear();
                }
            },
            ADDR_ENV,
            &mut String::new()
        ));
    }

    let mut job: Job = cmd.into();
    enum Id {
        Job,
        Str,
    }
    let mut inp_jobs: Vec<(Id, usize)> = vec![];

    let inp_redirs: RV = (0..inp_redir_n)
        .map(|i| vm.arg_get_val(nargs + 3 + 1 + i))
        .collect();
    let inp_redirs = te!(inp_redirs);

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

type Addr = fn(&Vm, usize) -> Result<(usize, usize)>;

const ADDR_ARG: Addr = |vm, i| Ok((te!(vm.arg_addr(i)), 1));

const ADDR_ENV: Addr = |vm, i| {
    let &nargs: &usize = te!(vm.arg_get(0));
    let &ninpredr: &usize = te!(vm.arg_get(nargs + 3));
    Ok((te!(vm.arg_addr(nargs + 3 + ninpredr + 1 + (i * 2))), 2))
};

fn install_args<G, I>(vm: &mut Vm, install: &mut I, argaddr: G, sbuf: &mut String) -> Result<()>
where
    G: Fn(&Vm, usize) -> Result<(usize, usize)>,
    I: FnMut(&str),
{
    let (nargs_addr, _) = te!(argaddr(vm, 0));
    let &nargs: &usize = te!(vm.stack_get(nargs_addr));
    ldebug!("injecting {} args from {}", nargs, nargs_addr);

    for i in 1..=nargs {
        let (addr, len) = te!(argaddr(vm, i));
        for j in 0..len {
            te!(inject_arg(vm, install, addr + j, sbuf));
        }
    }
    Ok(())
}

fn inject_arg<I>(vm: &mut Vm, inject: &mut I, arg_addr: usize, sbuf: &mut String) -> Result<()>
where
    I: FnMut(&str),
{
    let arg: &Value = vm.stack_get_val(arg_addr);
    match arg {
        &Value::LitString(value::LitString(strid)) => {
            let arg: &str = te!(vm.get_string_id(strid));
            inject(arg);
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                te!(inject_arg(vm, inject, ptr - i, sbuf));
            }
        }
        Value::Natural(n) => {
            sbuf.clear();
            te!(write!(sbuf, "{}", n));
            inject(sbuf);
        }
        &Value::Job(value::Job(jobid)) => {
            let job = te!(vm.get_job_mut(jobid));
            inject(te!(job.make_string()));
        }
        Value::DynString(value::DynString(string)) => {
            inject(string);
        }
        other => temg!("Cannot inject arg@{}: {:?}", arg_addr, other),
    }
    Ok(())
}
