use {
    super::CallArgs,
    crate::{te, temg, Result, Value, Vm},
    std::io,
};

pub fn builtin(vm: &mut Vm) -> Result<()> {
    let call_args = te!(CallArgs::<&Value>::from_vm(vm));
    todo!()
}

pub fn to_shell(call_args: CallArgs<&Value>) -> Result<()> {
    todo!()
}
