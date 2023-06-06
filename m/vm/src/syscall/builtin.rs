use {
    super::CallArgs,
    crate::{te, temg, Result, Value, Vm},
    std::io,
};

pub fn builtin(vm: &mut Vm) -> Result<()> {
    let _r = vm.prepare_call();
    te!(_r);

    let call_args = te!(CallArgs::<&Value>::from_vm(vm));
    log::debug!("[builtin::args] {call_args:?}");

    let _r = call_args.vmargs(vm);
    let vmargs: Vec<&Value> = te!(_r);

    log::debug!("[builtin::vmargs] {vmargs:?}");

    todo!()
}

pub fn to_shell(call_args: CallArgs<&Value>) -> Result<()> {
    todo!()
}
