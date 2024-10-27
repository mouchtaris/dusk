use {
    super::{ldebug, te, temg, value, Job, Result, Value, Vm},
    std::{
        fmt::Write,
        process::{Command, Stdio},
    },
};

pub fn spawn(vm: &mut Vm) -> Result<()> {
    te!(vm.prepare_call());

    let &nargs: &usize = te!(vm.arg_get(0));
    let cwd: &Value = te!(vm.arg_get_val(nargs + 1));
    let target: &Value = te!(vm.arg_get_val(nargs + 2));
    let &inp_redir_n: &usize = te!(vm.arg_get(nargs + 3));
    let &nenvs: &usize = te!(vm.arg_get(nargs + 3 + inp_redir_n + 1));

    let vmargs: Result<Vec<_>> = (0..=nargs).map(|i| vm.arg_get_val(i)).collect();
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

    // Set command name and close stdin
    //
    let mut cmd = Command::new(target);
    cmd.stdin(Stdio::null());

    if !cwd.is_null() {
        let cwd = cwd.to_owned();
        te!(inject_val(vm, &cwd, &mut |cwd| {
            error::ltrace!("cwd = '{}'", cwd);
            cmd.current_dir(cwd);
        }));
    }

    // Set command args
    //
    te!(install_args(
        vm,
        &mut |arg| {
            cmd.arg(arg);
        },
        ADDR_ARG,
        &mut String::new()
    ));

    // Set command environment
    //
    {
        let mut value = String::new();
        te!(install_args(
            vm,
            &mut |env| {
                if value.is_empty() {
                    value.push_str(env);
                } else {
                    cmd.env(env, value.as_str());
                    value.clear();
                }
            },
            ADDR_ENV,
            &mut String::new()
        ));
    }

    // Turn into a job
    //
    let mut job: Job = cmd.into();

    //
    // ## Connect input redirections ##
    //
    // ---
    // A "Job ID" that can be a Job, String or DynString id,
    // refering to the respective object in the vm.
    //
    // The commonality among these is that they can all be used
    // as a source to connect to a process' stdin.
    #[derive(Debug)]
    enum Id {
        Job,
        Str,
        DStr,
        Nat,
        Null,
    }
    //
    // ---
    // Extract (Id, id) from the arguments on the stack.
    //
    let mut inp_jobs: Vec<(Id, usize)> = vec![];
    //
    // Collect the values from the stack.
    let inp_redirs: Result<Vec<Value>> = (0..inp_redir_n)
        .rev() // as always, pushed values are reverse (same as arguments)
        .map(|i| vm.arg_get_val(nargs + 3 + 1 + i).map(<_>::to_owned))
        .collect();
    let inp_redirs = te!(inp_redirs);
    //
    // Translate each value to (Id, id).
    for mut redir in inp_redirs {
        let redir_insert = loop {
            // An input redirection value can only be one of the value types
            // defined here.
            break match redir {
                Value::Job(value::Job(jobid)) => (Id::Job, jobid),
                Value::LitString(value::LitString(strid)) => (Id::Str, strid),
                Value::DynString(value::DynString(strid)) => (Id::DStr, strid),
                Value::Natural(n) => (Id::Nat, n),
                Value::ArrayView(view) => {
                    // An array view (slice) means the first element of it.
                    redir = te!(view.first(vm));
                    if redir.is_null() && matches!(view.end, value::Signed::Minus(0)) {
                        // As a special case, an ending, var-arg slice (`[n;]`)
                        // can be permitted as a NULL Input Redirection Source.
                        (Id::Null, 0)
                    } else {
                        continue;
                    }
                }
                other => temg!("Invalid value as source in input redirection: {:?}", other),
            };
        };
        inp_jobs.push(redir_insert)
    }
    //
    // Create a Job from a text string source type. When connected it will
    // output the string value itself.
    fn make_string_input(vm: &Vm, typ: Id, id: usize) -> Result<Job> {
        Ok({
            let cmd = Command::new("<internal string source>");
            let string = match typ {
                Id::Str => te!(vm.get_string_id(id)).to_owned(),
                Id::DStr => te!(vm.get_dynstring_id(id)).to_owned(),
                Id::Nat => format!("{}", id),
                Id::Null => String::new(),
                other => temg!("Not a string: {:?}", other),
            };
            let buffer = job::Buffer::String(cmd, string);
            Job::Buffer(buffer)
        })
    }
    //
    // Get a Job object from each (Id, id) defined as input, and add them
    // to the target Job's input list.
    for inp_job0 in inp_jobs {
        match inp_job0 {
            (Id::Job, jobid) => {
                let inp_job = te!(vm.get_job_mut(jobid));
                te!(job.add_input_job(inp_job));
            }
            (typ @ (Id::Str | Id::DStr | Id::Nat | Id::Null), strid) => {
                let mut inp_job = te!(make_string_input(vm, typ, strid));
                te!(job.add_input_job(&mut inp_job));
            }
        }
    }

    // Add job to job table and get its ID.
    //
    let job_id = vm.add_job(job);

    // Hand-handle stack frame
    //
    // - Allocate 1 local, to push a job-id value
    vm.allocate(1);
    let val: Value = value::Job(job_id).into();
    ldebug!("put {:?} to {}", val, vm.stackp());
    te!(vm.wait_debugger(format_args!("{:?}", val)));
    // - Push the local
    te!(vm.push_val(val));
    // - Set-ret-val from the local
    te!(vm.set_ret_val_from_local(0));
    // - Return from call
    te!(vm.return_from_call(0));
    // - Dealloc the 1
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
    let arg: &Value = te!(vm.stack_get_val(arg_addr));
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
        &Value::DynString(value::DynString(string_id)) => {
            let string = te!(vm.get_dynstring_id(string_id));
            inject(string);
        }
        &Value::ArrayView(view) => {
            for val in te!(view.collect_all(vm, &mut <_>::default())) {
                te!(inject_val(vm, val, inject))
            }
        }
        other => temg!("Cannot inject arg@{}: {:?}", arg_addr, other),
    }
    Ok(())
}

fn inject_val<I>(vm: &mut Vm, val: &Value, inject: &mut I) -> Result<()>
where
    I: FnMut(&str),
{
    match val {
        &Value::LitString(value::LitString(strid)) => {
            let val: &str = te!(vm.get_string_id(strid));
            inject(val);
        }
        Value::Natural(n) => {
            let mut sbuf = [0u8; 64];
            use {std::io::Write, std::str::from_utf8};
            te!(write!(sbuf.as_mut_slice(), "{}", n));
            let end = sbuf
                .as_slice()
                .iter()
                .cloned()
                .position(|b| b == 0)
                .unwrap_or(0);
            let subsl = &sbuf.as_slice()[0..end as usize];
            inject(te!(from_utf8(subsl)));
        }
        &Value::Job(value::Job(jobid)) => {
            let job = te!(vm.get_job_mut(jobid));
            inject(te!(job.make_string()));
        }
        &Value::DynString(value::DynString(string_id)) => {
            let string = te!(vm.get_dynstring_id(string_id));
            inject(string);
        }
        &Value::Array(value::Array { ptr }) => {
            let &arrlen: &usize = te!(vm.stack_get(ptr));
            for i in 1..=arrlen {
                let val = te!(vm.stack_get_val(ptr + i)).to_owned();
                te!(inject_val(vm, &val, inject));
            }
        }
        Value::ArrayView(view) => {
            te!(view.forall(vm, |vm, val| inject_val(vm, val, inject)));
        }
        other => temg!("Not supported as to-string: {:?}", other),
    }
    Ok(())
}
