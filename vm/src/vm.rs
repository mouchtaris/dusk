use {
    super::{
        ldebug, ltrace, soft_todo, te, Deq, ICode, Instr, Result, StringInfo, TryFrom, Value,
        ValueTypeInfo,
    },
    std::{borrow::Borrow, fmt, mem},
};

#[derive(Default, Debug)]
pub struct Vm {
    pub bin_path: Deq<String>,
    retval: Value,
    string_table: Deq<String>,
    stack: Vec<Value>,
    frame_pointer: usize,
}

impl Vm {
    fn add_string(&mut self, i: StringInfo, s: String) {
        let t = &mut self.string_table;
        let id = i.id;
        if id < t.len() {
            t[id] = s;
        } else {
            t.resize(id + 1, s);
        }
    }
    /// Put current retval to recycle bins and return reference to fresh
    /// Return the current retval as it is and reset retval.
    fn take_retval(&mut self) -> Value {
        mem::take(&mut self.retval)
    }

    fn frame_addr(&self, offset: usize) -> usize {
        self.frame_pointer + offset
    }

    /// Reset to init state
    pub fn reset(&mut self) {
        self.string_table.clear();

        self.frame_pointer = 0;
        self.stack.clear();
    }

    /// Take retval, converting to underlying type
    pub fn rv_take<T>(&mut self) -> Result<T>
    where
        T: TryFrom<Value, Error = Value> + ValueTypeInfo,
    {
        self.take_retval().try_into()
    }
    /// Borrow retval, converting to mut ref to underlying type
    pub fn rv_mut<T>(&mut self) -> Result<&mut T>
    where
        for<'s> &'s mut T: TryFrom<&'s mut Value, Error = &'s mut Value>,
        T: ValueTypeInfo,
    {
        (&mut self.retval).try_mut()
    }

    /// Set a value to offset from stack-frame-pointer
    pub fn frame_set<V>(&mut self, offset: usize, v: V)
    where
        V: Into<Value> + ValueTypeInfo,
    {
        let addr = self.frame_addr(offset);
        let value = v.into();
        ltrace!("frame[{}] = {:?}", addr, value);
        *self.stack.get_mut(addr).unwrap() = value;
    }
    /// Take the value from stack-frame-offset
    pub fn frame_take<T>(&mut self, offset: usize) -> Result<T>
    where
        T: TryFrom<Value, Error = Value> + ValueTypeInfo + fmt::Debug,
    {
        let addr = self.frame_addr(offset);
        let r = mem::take(te!(self.stack.get_mut(addr))).try_into();
        ltrace!("Reading stack {}: {:?}", addr, r);
        r
    }
    /// Borrow retval, converting to mut ref to underlying type
    pub fn frame_mut<T>(&mut self, offset: usize) -> Result<&mut T>
    where
        for<'s> &'s mut T: TryFrom<&'s mut Value, Error = &'s mut Value> + fmt::Debug,
        T: ValueTypeInfo,
    {
        let addr = self.frame_addr(offset);
        let r = te!(self.stack.get_mut(addr)).try_mut();
        ltrace!("Reading mut stack {}: {:?}", addr, r);
        r
    }

    /// ! Assumes reset has been done !
    /// Set retval. Does not recycle.
    pub fn set_retval<V>(&mut self, v: V)
    where
        V: Into<Value> + ValueTypeInfo,
    {
        ltrace!("value = {}", V::type_info_name());
        self.retval = v.into();
    }

    /// Set frame-offset to String, initializing from `string_table[strid]`
    pub fn load_string(&mut self, strid: usize, dst: usize) -> &mut String {
        let src = self.string_table[strid].to_owned();
        ldebug!("Loading string {}: {}", strid, src);
        self.frame_set(dst, src);
        self.frame_mut(dst).unwrap()
    }

    /// Grow the stack by `size`
    pub fn allocate(&mut self, size: usize) {
        self.stack
            .resize_with(self.stack.len() + size, <_>::default);
    }

    pub fn load_icode(mut self, icode: &ICode) -> Result<Self> {
        for (s, i) in &icode.strings {
            ltrace!("Load literal string {} {}", i.id, s);
            self.add_string(i.clone(), s.clone());
        }
        self = te!(self.run_instructions(&icode.instructions));
        Ok(self)
    }

    pub fn run_instructions<I>(mut self, icode: I) -> Result<Self>
    where
        I: IntoIterator,
        I::Item: Borrow<Instr>,
    {
        for instruction in icode {
            self = te!(instruction.borrow().operate_on(self));
        }
        Ok(self)
    }

    pub fn init_bin_path<S>(&mut self, source: S)
    where
        S: AsRef<str>,
    {
        let vm = self;
        vm.bin_path = source.as_ref().split(':').map(<_>::to_owned).collect();
    }
    pub fn init_bin_path_from_env<S>(&mut self, env_name: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        Ok(self.init_bin_path(te!(std::env::var(env_name.as_ref()))))
    }
    pub fn init_bin_path_from_path_env(&mut self) -> Result<()> {
        self.init_bin_path_from_env("PATH")
    }
}
