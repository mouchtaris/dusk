use {
    super::{ltrace, te, Deq, ICode, Instr, Result, StringInfo, TryFrom, Value, ValueTypeInfo},
    std::{borrow::Borrow, fmt, io, mem},
};

#[derive(Default, Debug)]
pub struct Vm {
    pub bin_path: Deq<String>,
    string_table: Deq<String>,
    stack: Vec<Value>,
    frame_ptr: usize,
    arg_ptr: usize,
    stack_ptr: usize,
    instr_ptr: usize,
}

impl Vm {
    /// Reset to init state
    pub fn reset(&mut self) {
        self.string_table.clear();

        self.frame_ptr = 0;
        self.stack_ptr = 0;
        self.arg_ptr = 0;
        self.stack.clear();
        self.instr_ptr = 0;
    }

    fn argp_next(&mut self) -> usize {
        let argp = self.arg_ptr;
        self.arg_ptr += 1;
        argp
    }

    /// Increase frame pointer
    fn frame_addr(&self, offset: usize) -> usize {
        self.frame_ptr + offset
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

    pub fn frame_take_value(&mut self, offset: usize) -> Result<Value> {
        let addr = self.frame_addr(offset);
        let r = mem::take(te!(self.stack.get_mut(addr)));
        ltrace!("Reading stack {}: {:?}", addr, r);
        Ok(r)
    }

    pub fn frame_take<T>(&mut self, offset: usize) -> Result<T>
    where
        T: TryFrom<Value, Error = Value> + ValueTypeInfo + fmt::Debug,
    {
        te!(self.frame_take_value(offset)).try_into()
    }

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

    pub fn frame<T>(&self, offset: usize) -> Result<&T>
    where
        for<'s> &'s T: TryFrom<&'s Value, Error = &'s Value> + fmt::Debug,
        T: ValueTypeInfo,
    {
        let addr = self.frame_addr(offset);
        let r = te!(self.stack.get(addr)).try_ref();
        ltrace!("Reading mut stack {}: {:?}", addr, r);
        r
    }

    pub fn push_val<T>(&mut self, src: T)
    where
        T: ValueTypeInfo + Into<Value>,
    {
        let addr = self.argp_next();
        self.frame_set(addr, src);
    }

    pub fn push_null(&mut self) {
        self.push_val(());
    }

    pub fn push_str(&mut self, strid: usize) {
        let src = self.string_table[strid].to_owned();
        self.push_val(src);
    }

    /// Grow the stack by `size`
    pub fn allocate(&mut self, size: usize) {
        self.stack
            .resize_with(self.stack.len() + size, <_>::default);
        self.stack_ptr += size;
    }

    pub fn load_icode(mut self, icode: &ICode) -> Result<Self> {
        for (s, i) in &icode.strings {
            ltrace!("Load literal string {} {}", i.id, s);
            self.add_string(i.clone(), s.clone());
        }
        self = te!(self.run_instructions(&icode.instructions));
        Ok(self)
    }

    pub fn run_instructions(mut self, icode: &Deq<Instr>) -> Result<Self> {
        while self.instr_ptr < icode.len() {
            let instruction = &icode[self.instr_ptr];
            self.instr_ptr += 1;
            self = te!(instruction.borrow().operate_on(self));
        }
        Ok(self)
    }

    pub fn jump(&mut self, addr: usize) {
        self.instr_ptr = addr;
    }

    pub fn init_bin_path<S>(&mut self, source: S)
    where
        S: AsRef<str>,
    {
        let vm = self;
        vm.bin_path = source.as_ref().split(':').map(<_>::to_owned).collect();
    }
    pub fn init_bin_path_system(&mut self) {
        self.init_bin_path("/sbin:/usr/sbin:/usr/local/sbin:/bin:/usr/bin:/usr/local/bin")
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

    pub fn write_to<O>(&self, o: io::Result<O>) -> io::Result<()>
    where
        O: io::Write,
    {
        o.and_then(|mut o| {
            Ok({
                writeln!(o, "=== BIN_PATH ===")?;
                for path in &self.bin_path {
                    writeln!(o, "> {}", path)?;
                }
                writeln!(o, "=== STRING TABLE ===")?;
                let mut i = 0;
                for string in &self.string_table {
                    writeln!(o, ">[{:4}] {:?}", i, string)?;
                    i += 1;
                }
                writeln!(o, "=== STACK ===")?;
                let mut i = 0;
                for cell in &self.stack {
                    writeln!(o, ">[{:4}] {:?}", i, cell)?;
                    i += 1;
                }
                writeln!(o, "=== STATE ===")?;
                writeln!(o, "- frame pointer    : {}", self.frame_ptr)?;
            })
        })
    }

    fn add_string(&mut self, i: StringInfo, s: String) {
        let t = &mut self.string_table;
        let id = i.id;
        if id < t.len() {
            t[id] = s;
        } else {
            t.resize(id + 1, s);
        }
    }
}
