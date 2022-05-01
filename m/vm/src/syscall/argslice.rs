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

    fn get_num(vm: &Vm, val: &Value) -> Result<value::Signed<u16>> {
        Ok(if let Ok(n) = val.try_ref::<value::Natural>() {
            value::Plus(*n as u16)
        } else if let Ok(&value::LitString(sid)) = val.try_ref::<value::LitString>() {
            use std::str::FromStr;

            match te!(vm.get_string_id(sid)) {
                s if s.starts_with('-') => value::Minus(te!(u16::from_str(&s[1..]), "{}", s)),
                s => value::Plus(te!(i16::from_str(s), "{}", s) as u16),
            }
        } else if let Ok(view) = val.try_ref::<value::ArrayView>() {
            te!(get_num(vm, te!(view.to_owned().first(vm))))
        } else {
            temg!("{:?}", val)
        })
    }

    let start = te!(get_num(vm, te!(vm.arg_get_val(1))));
    let end = te!(get_num(vm, te!(vm.arg_get_val(2))));

    let args = te!(vm.arg_get_val_mut(3));
    let mut slice0: value::ArrayView = <_>::default();
    let args: value::ArrayView = match args {
        &mut Value::Array(arr) => value::ArrayView::new(arr, start, end),
        &mut Value::ArrayView(mut slice) => {
            te!(slice.arrlen(vm));
            slice0 = slice.to_owned();
            te!(from_slice(&slice, start, end))
        }
        other => temg!("{:?}", other),
    };

    ldebug!("argslice {}[{}..{}] = {}", slice0, start, end, args,);

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
    start: value::Signed<u16>,
    end: value::Signed<u16>,
) -> Result<value::ArrayView> {
    use value::{Minus, Plus};

    let &value::ArrayView { arr, len, .. } = slice;

    let start = match start {
        Plus(a) => match slice.start {
            Plus(b) => Plus(b + a),
            Minus(b) => Minus(b - a),
        },
        Minus(a) => match slice.end {
            Plus(b) => Plus(b - a),
            Minus(b) => Minus(b + a),
        },
    };
    let end = match end {
        Plus(a) => match slice.start {
            Plus(b) => Plus(b + a),
            Minus(b) => Minus(b - a),
        },
        Minus(a) => match slice.end {
            Plus(b) => Plus(b - a),
            Minus(b) => Minus(b + a),
        },
    };

    Ok(value::ArrayView {
        arr,
        start,
        end,
        len,
    })
}
