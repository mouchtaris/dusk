use {
    super::{value, Job, Result, Value, Vm},
    error::{ldebug, te, temg},
    std::borrow::Borrow,
};

pub const SPAWN: usize = usize::MAX;
pub const ARG_SLICE: usize = usize::MAX - 1;

mod argslice;
mod builtin;
mod spawn;
pub use {argslice::argslice, builtin::builtin, spawn::spawn};

/// An addressor function; translates a conceptual index (for example:
/// an argument index, input redirection index, environment setting index,
/// etc), into an absolute stack address, alongside it's length on the
/// stack (1 cell, 2 cells, etc).
type Addr = fn(&Vm, usize) -> Result<(usize, usize)>;

/// Translate as an argument index.
/// Arguments are always length 1.
const ADDR_ARG: Addr = |vm, i| Ok((te!(vm.arg_addr(i)), 1));

/// Translate as an environment setting.
/// Env-settings are always length 2.
const ADDR_ENV: Addr = |vm, i| {
    let &nargs: &usize = te!(vm.arg_get(0));
    let &ninpredr: &usize = te!(vm.arg_get(nargs + 3));
    Ok((te!(vm.arg_addr(nargs + 3 + ninpredr + 1 + (i * 2))), 2))
};

/// Translate a vm Value into a string value, and call the given callback
/// with it.
///
/// This is useful on one hand to translate all the various types of
/// vm::Value-s into text, but also to expand compound values (such as
/// slices, arrays, etc).
pub fn inject_val<I>(vm: &mut Vm, val: &Value, inject: &mut I) -> Result<()>
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

pub struct CallArgs<Value> {
    nargs: usize,
    nenvs: usize,
    ninps: usize,
    nouts: usize,
    targt: Value,
    cwd: Value,
}

impl<V> CallArgs<V> {
    fn from_vm(vm: &Vm) -> Result<CallArgs<&Value>> {
        let &nargs: &usize = te!(vm.arg_get(0));
        let cwd: &Value = te!(vm.arg_get_val(nargs + 1));
        let target: &Value = te!(vm.arg_get_val(nargs + 2));
        let &inp_redir_n: &usize = te!(vm.arg_get(nargs + 3));
        let &nenvs: &usize = te!(vm.arg_get(nargs + 3 + inp_redir_n + 1));

        let call_args = CallArgs {
            nargs,
            nenvs,
            ninps: inp_redir_n,
            nouts: 0,
            targt: target,
            cwd,
        };
        Ok(call_args)
    }
    fn to_owned(&self) -> CallArgs<V::Owned>
    where
        V: ToOwned,
    {
        let &Self {
            nargs,
            nenvs,
            ninps,
            nouts,
            ..
        } = self;
        let Self { targt, cwd, .. } = self;
        let targt = targt.to_owned();
        let cwd = cwd.to_owned();
        CallArgs {
            nargs,
            nenvs,
            ninps,
            nouts,
            targt: targt,
            cwd: cwd,
        }
    }
}
