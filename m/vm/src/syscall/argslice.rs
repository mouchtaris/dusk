use {
    super::{value, Result, Value, Vm},
    error::{ldebug, te, temg},
};
pub fn argslice(vm: &mut Vm) -> Result<()> {
    te!(vm.prepare_call());

    let &nargs: &usize = te!(vm.arg_get(0));

    if nargs < 3 {
        temg!("argslice START END ARGS: nargs={}", nargs);
    }

    fn get_num(vm: &mut Vm, val: &mut Value) -> Result<value::Signed<u16>> {
        Ok(if let Ok(n) = val.try_ref::<value::Natural>() {
            value::Plus(*n as u16)
        } else if let Ok(&value::LitString(sid)) = val.try_ref::<value::LitString>() {
            use std::str::FromStr;

            match te!(vm.get_string_id(sid)) {
                s if s.starts_with('-') => value::Minus(te!(u16::from_str(&s[1..]), "{}", s)),
                s => value::Plus(te!(i16::from_str(s), "{}", s) as u16),
            }
        } else if let Ok(view) = val.try_mut::<value::ArrayView>() {
            let mut val = te!(view.to_owned().first(vm));
            te!(get_num(vm, &mut val))
        } else {
            temg!("{:?}", val)
        })
    }

    let mut start_val = te!(vm.arg_get_val(1)).to_owned();
    let mut end_val = te!(vm.arg_get_val(2)).to_owned();

    let start = te!(get_num(vm, &mut start_val));
    let end = te!(get_num(vm, &mut end_val));

    let args = te!(vm.arg_get_val_mut(3));
    let args: value::ArrayView = match args {
        &mut Value::Array(arr) => {
            ldebug!("from {:?}", arr);
            value::ArrayView::new(arr, start, end)
        }
        &mut Value::ArrayView(slice) => {
            ldebug!("from {:?}", slice);
            te!(from_slice(&slice, start, end))
        }
        other => temg!("Invalid argslice arg3: {:?}", other),
    };

    ldebug!("argslice [{}..{}] = {}", start, end, args,);

    vm.allocate(1);
    te!(vm.wait_debugger(format_args!("{:?}", args)));
    te!(vm.push_val(args));
    te!(vm.set_ret_val_from_local(0));
    te!(vm.return_from_call(0));
    vm.dealloc(1);
    Ok(())
}

fn from_slice(
    slice: &value::ArrayView,
    rstart: value::Signed<u16>,
    rend: value::Signed<u16>,
) -> Result<value::ArrayView> {
    let &value::ArrayView { arr, start, end } = slice;

    let r = (start, end);
    let start = rstart.rebase(r);
    let end = rend.rebase(r);

    Ok(value::ArrayView { arr, start, end })
}
