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

    fn stackp_next(&mut self) -> usize {
        let stackp = self.stack_ptr;
        self.stack_ptr += 1;
        ltrace!("stackp++ {}", stackp);
        stackp
    }

    fn frame_addr(&self, offset: usize) -> usize {
        self.frame_ptr + offset
    }

    fn arg_addr(&self, argn: usize) -> usize {
        // -n for stack-frame info: retaddr etc
        // -1 because fp points 1 beyond last
        self.frame_ptr - 1 - self.call_stack_data().len() - argn
    }

    fn call_stack_data(&self) -> [usize; 2] {
        [self.frame_ptr, self.instr_ptr]
    }
    pub fn ret_instr_addr(&self) -> usize {
        *self.stack_get(self.frame_ptr - 1).unwrap()
    }
    pub fn ret_fp_addr(&self) -> usize {
        *self.stack_get(self.frame_ptr - 2).unwrap()
    }
    pub fn prepare_call(&mut self) {
        let vm = self;

        let frame = vm.call_stack_data();
        vm.allocate(frame.len());
        frame.iter().for_each(|&v| vm.push_val(v));
        ltrace!("save stack [fp, rti]: {:?}", frame);

        vm.frame_ptr = vm.stack_ptr;
    }
    pub fn return_from_call(&mut self) {
        let vm = self;
        let ret_instr = vm.ret_instr_addr();
        let ret_fp = vm.ret_fp_addr();
        ltrace!("return fp[{}] inst[{}]", ret_fp, ret_instr);

        vm.dealloc(vm.call_stack_data().len());
        vm.frame_ptr = ret_fp;

        vm.jump(ret_instr);
    }

    pub fn stack_get_val(&self, addr: usize) -> &Value {
        &self.stack[addr]
    }
    pub fn stack_get_val_mut(&mut self, addr: usize) -> &mut Value {
        if addr < self.stack.len() {
            self.stack.get_mut(addr).unwrap()
        } else {
            eprintln!("== vm ==");
            self.write_to(Ok(std::io::stderr())).unwrap_or(());
            panic!("invalid stack access {}[{}]", self.stack.len(), addr);
        }
    }
    pub fn stack_take_val(&mut self, addr: usize) -> Value {
        mem::take(self.stack_get_val_mut(addr))
    }
    pub fn stack_set<V: Into<Value>>(&mut self, addr: usize, val: V) {
        let cell = self.stack_get_val_mut(addr);
        *cell = val.into();
    }
    pub fn stack_get<T>(&self, addr: usize) -> Result<&T>
    where
        for<'r> &'r T: TryFrom<&'r Value, Error = &'r Value> + ValueTypeInfo,
    {
        self.stack_get_val(addr).try_ref()
    }
    pub fn stack_get_mut<T>(&mut self, addr: usize) -> Result<&mut T>
    where
        for<'r> &'r mut T: TryFrom<&'r mut Value, Error = &'r mut Value> + ValueTypeInfo,
    {
        self.stack_get_val_mut(addr).try_mut()
    }
    pub fn stack_take<T>(&mut self, addr: usize) -> Result<T>
    where
        T: TryFrom<Value, Error = Value> + ValueTypeInfo,
    {
        self.stack_take_val(addr).try_into()
    }

    pub fn frame_get_val(&self, offset: usize) -> &Value {
        self.stack_get_val(self.frame_addr(offset))
    }
    pub fn frame_get_val_mut(&mut self, offset: usize) -> &mut Value {
        self.stack_get_val_mut(self.frame_addr(offset))
    }
    pub fn frame_take_val(&mut self, offset: usize) -> Value {
        self.stack_take_val(self.frame_addr(offset))
    }
    pub fn frame_set<V>(&mut self, offset: usize, v: V)
    where
        V: Into<Value> + ValueTypeInfo,
    {
        self.stack_set(self.frame_addr(offset), v)
    }
    pub fn frame_get<T>(&self, offset: usize) -> Result<&T>
    where
        for<'s> &'s T: TryFrom<&'s Value, Error = &'s Value> + ValueTypeInfo,
    {
        self.frame_get_val(offset).try_ref()
    }
    pub fn frame_get_mut<T>(&mut self, offset: usize) -> Result<&mut T>
    where
        for<'s> &'s mut T: TryFrom<&'s mut Value, Error = &'s mut Value> + ValueTypeInfo,
    {
        self.frame_get_val_mut(offset).try_mut()
    }
    pub fn frame_take<T>(&mut self, offset: usize) -> Result<T>
    where
        T: TryFrom<Value, Error = Value> + ValueTypeInfo + fmt::Debug,
    {
        self.frame_take_val(offset).try_into()
    }

    pub fn arg_get_val(&self, argn: usize) -> &Value {
        self.stack_get_val(self.arg_addr(argn))
    }
    pub fn arg_get_val_mut(&mut self, argn: usize) -> &mut Value {
        self.stack_get_val_mut(self.arg_addr(argn))
    }
    pub fn arg_get<T>(&self, argn: usize) -> Result<&T>
    where
        for<'s> &'s T: TryFrom<&'s Value, Error = &'s Value> + ValueTypeInfo,
    {
        self.arg_get_val(argn).try_ref()
    }
    pub fn arg_get_mut<T>(&mut self, argn: usize) -> Result<&mut T>
    where
        for<'s> &'s mut T: TryFrom<&'s mut Value, Error = &'s mut Value> + ValueTypeInfo,
    {
        self.arg_get_val_mut(argn).try_mut()
    }

    pub fn push_val<T>(&mut self, src: T)
    where
        T: ValueTypeInfo + Into<Value>,
    {
        let addr = self.stackp_next();
        self.stack_set(addr, src);
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
        ltrace!("alloc: {}", size);
        self.stack
            .resize_with(self.stack.len() + size, <_>::default);
    }

    pub fn dealloc(&mut self, size: usize) {
        self.stack.shrink_to(size);
        self.stack_ptr -= size;
        ltrace!("stackp - {} = {}", size, self.stack_ptr);
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

    /// Set the next instr_ptr to be executed
    pub fn jump(&mut self, addr: usize) {
        ltrace!("[jump] {}", addr);
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
                //writeln!(o, "=== BIN_PATH ===")?;
                //for path in &self.bin_path {
                //    writeln!(o, "> {}", path)?;
                //}
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
                writeln!(o, "- arg   pointer    : {}", self.arg_ptr)?;
                writeln!(o, "- stack pointer    : {}", self.stack_ptr)?;
                writeln!(o, "- instr pointer    : {}", self.instr_ptr)?;
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
