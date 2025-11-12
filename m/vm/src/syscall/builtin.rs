use {
    super::CallArgs,
    crate::{te, temg, Result, Value, Vm},
};

pub fn builtin(vm: &mut Vm) -> Result<()> {
    let _r = vm.prepare_call();
    te!(_r);

    let call_args = te!(CallArgs::<&Value>::from_vm(vm));
    log::debug!("[builtin::args] {call_args:?}");

    let vmargs: Vec<&Value> = call_args.args;

    log::debug!("[builtin::vmargs] {vmargs:?}");

    let builtin_name: &Value = te!(vmargs.get(1), "Missing builtin name");
    let builtin_name: &str = te!(vm.val_as_str(builtin_name));
    let builtin_name: String = builtin_name.to_owned();

    {
        let dbg_desc = format!("__builtin ---- {builtin_name} ----");
        te!(vm.wait_debugger(dbg_desc));
    }

    let retval: Value = te!(match builtin_name.as_str() {
        "__lib" => GET_VM_ICODE(vm),
        other => temg!("Unknown builtin: {other}"),
    });

    te!(vm.set_ret_val(retval));
    te!(vm.return_from_call2());
    Ok(())
}

pub struct BuiltinArgs<'a>(&'a Vm, pub CallArgs<&'a Value>);
impl<'s> BuiltinArgs<'s> {
    pub fn from_vm(vm: &'s mut Vm) -> Result<Self> {
        let call_args: CallArgs<&Value> = te!(CallArgs::<&Value>::from_vm(vm));
        Ok(Self(vm, call_args))
    }
    pub fn nargs(&self) -> Result<usize> {
        let val: &Value = te!(self.1.args.get(0));
        let size = te!(val.as_number());
        Ok(size)
    }
    pub fn builtin_name(&self) -> Result<&str> {
        let val: &Value = te!(self.1.args.get(1));
        let name = te!(self.0.val_as_str(val));
        Ok(name)
    }
    pub fn arg(&self, idx: usize) -> Result<&Value> {
        Ok(te!(self.1.args.get(2 + idx)))
    }
    pub fn arg_str(&self, idx: usize) -> Result<&str> {
        Ok(te!(self.0.val_as_str(te!(self.arg(idx)))))
    }
}

type SysCall = fn(&mut Vm) -> Result<Value>;

const GET_VM_ICODE: SysCall = |vm| {
    let script = te!(vm.current_script_value());
    let val: Value = script.to_owned();
    Ok(val)
};

pub fn to_shell(call_args: CallArgs<&Value>) -> Result<()> {
    todo!()
}
