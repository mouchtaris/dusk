use {
    crate::{
        value::{Array, Minus, Plus, Signed, Value},
        Result, Vm,
    },
    error::te,
    std::fmt,
};

#[derive(Copy, Clone, Default)]
pub struct ArrayView {
    pub start: Signed<u16>,
    pub end: Signed<u16>,
    pub arr: Array,
}

impl ArrayView {
    pub fn new(arr: Array, start: Signed<u16>, end: Signed<u16>) -> Self {
        Self { arr, start, end }
    }

    pub fn arr_all(arr: Array) -> Self {
        Self::new(arr, Plus(0), Minus(0))
    }

    pub fn expand_all<C>(&self, vm: &mut Vm, callb: &mut C) -> Result<Value>
    where
        C: FnMut(&mut Vm, &Value) -> Result<bool>,
    {
        let &Self { arr, .. } = self;
        let &len: &usize = te!(vm.stack_get(arr.ptr));
        let mut last_val = Value::default();
        for i in 1..=len {
            let val = te!(vm.stack_get_val(arr.ptr - i)).to_owned();
            if !te!(expand_value(vm, val, &mut last_val, callb)) {
                break;
            }
        }
        Ok(last_val)
    }

    pub fn count_all(&self, vm: &mut Vm) -> Result<u16> {
        let mut count = 0;
        te!(self.expand_all(vm, &mut |_, _| {
            count += 1;
            Ok(true)
        }));
        Ok(count)
    }

    pub fn foreach<C>(&self, vm: &mut Vm, callb: &mut C) -> Result<Value>
    where
        C: FnMut(&mut Vm, &Value) -> Result<bool>,
    {
        let &Self { start, end, .. } = self;
        let len = te!(self.count_all(vm));
        let r = (0, len);
        let mut start = start.wrap(r);
        let mut end = end.wrap(r);
        if start == end {
            return Ok(<_>::default());
        }
        self.expand_all(vm, &mut |vm, val| {
            if end == 0 {
                return Ok(false);
            }
            end -= 1;

            if start == 0 {
                if !te!(callb(vm, val)) {
                    return Ok(false);
                }
            } else {
                start -= 1;
            }

            Ok(true)
        })
    }

    pub fn forall<C>(&self, vm: &mut Vm, mut callb: C) -> Result<()>
    where
        C: FnMut(&mut Vm, &Value) -> Result<()>,
    {
        te!(self.foreach(vm, &mut |vm, val| {
            te!(callb(vm, val));
            Ok(true)
        }));
        Ok(())
    }

    pub fn first(&self, vm: &mut Vm) -> Result<Value> {
        self.foreach(vm, &mut |_, _| Ok(false))
    }

    pub fn collect_all<'c>(
        &self,
        vm: &mut Vm,
        callb: &'c mut Vec<Value>,
    ) -> Result<&'c mut Vec<Value>> {
        te!(self.forall(vm, |_, val| Ok(callb.push(val.to_owned()))));
        Ok(callb)
    }
}

impl fmt::Display for ArrayView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}){{{}..{}}}", self.arr.ptr, self.start, self.end)?;
        Ok(())
    }
}

impl fmt::Debug for ArrayView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)?;
        Ok(())
    }
}

fn expand_view<C>(vm: &mut Vm, view: ArrayView, last_val: &mut Value, callb: &mut C) -> Result<bool>
where
    C: FnMut(&mut Vm, &Value) -> Result<bool>,
{
    // TODO: fight infinite recursion in compiler
    //let mut reenter = true;
    //last_val = te!(view.foreach(vm, {
    //    &mut |vm, val| {
    //        if !te!(callb(vm, &val.to_owned())) {
    //            reenter = false;
    //        }
    //        Ok(reenter)
    //    }
    //}));
    let mut all = Vec::new();
    te!(view.collect_all(vm, &mut all));

    for val in all {
        *last_val = val;
        if !te!(callb(vm, &last_val)) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn expand_value<C>(vm: &mut Vm, val: Value, last_val: &mut Value, callb: &mut C) -> Result<bool>
where
    C: FnMut(&mut Vm, &Value) -> Result<bool>,
{
    match val {
        Value::Array(arr) => expand_view(vm, ArrayView::arr_all(arr), last_val, callb),
        Value::ArrayView(view) => expand_view(vm, view, last_val, callb),
        val => {
            *last_val = val;
            callb(vm, &last_val)
        }
    }
}
