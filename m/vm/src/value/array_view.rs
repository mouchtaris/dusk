use {
    crate::{
        value::{Array, Minus, Plus, Signed, Value},
        Result, Vm,
    },
    error::te,
    std::fmt,
};

#[derive(Copy, Clone, Debug, Default)]
pub struct ArrayView {
    pub start: Signed<u16>,
    pub end: Signed<u16>,
    pub arr: Array,
    pub len: Option<u16>,
}

impl ArrayView {
    pub fn new(arr: Array, start: Signed<u16>, end: Signed<u16>) -> Self {
        Self {
            arr,
            len: <_>::default(),
            start,
            end,
        }
    }

    pub fn first<'v>(&mut self, vm: &'v Vm) -> Result<&'v Value> {
        te!(self.arrlen(vm));
        let i = te!(self.to_range()).start;
        vm.stack_get_val(self.ptr(i))
    }

    pub fn ptr(&self, i: u16) -> usize {
        self.arr.ptr - (i as usize)
    }

    pub fn arrlen(&mut self, vm: &Vm) -> Result<u16> {
        let Self {
            arr: Array { ptr },
            len,
            ..
        } = self;
        Ok(match len {
            l @ None => {
                let n = *te!(vm.stack_get::<usize>(*ptr)) as u16;
                *l = Some(n);
                n
            }
            &mut Some(n) => n,
        })
    }

    pub fn to_off(&self, i: &Signed<u16>) -> Result<u16> {
        let Self { len, .. } = self;
        let len = te!(len.to_owned());

        Ok(match i {
            &Plus(n) => n + 1,
            &Minus(n) => len + 1 - n,
        })
    }

    pub fn to_range(&self) -> Result<std::ops::Range<u16>> {
        let Self { start, end, .. } = self;
        let start: u16 = te!(self.to_off(start));
        let end: u16 = te!(self.to_off(end));

        Ok(start..end)
    }
}
impl fmt::Display for ArrayView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}:{}){{{}..{}}}",
            self.arr.ptr,
            self.len.unwrap_or(0),
            self.start,
            self.end
        )?;
        Ok(())
    }
}
