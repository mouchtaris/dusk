use {
    super::{
        debugger::Bugger as Debugger, ltrace, te, temg, value, Deq, ICode, Job, Result, StringInfo,
        TryFrom, Value, ValueTypeInfo,
    },
    std::{borrow::Borrow, io, mem, result},
};

#[derive(Default, Debug)]
pub struct Vm {
    pub bin_path: Deq<String>,
    string_table: Deq<String>,
    job_table: Deq<Job>,
    stack: Vec<Value>,
    frame_ptr: usize,
    stack_ptr: usize,
    instr_ptr: usize,
}

pub struct Stack {
    mem: Vec<Value>,
    fp: usize,
    sp: usize,
}
impl Stack {
    // TODO
}

impl Vm {
    /// Reset to zero state
    pub fn reset(&mut self) {
        self.string_table.clear();

        self.frame_ptr = 0;
        self.stack_ptr = 0;
        self.stack.clear();
        self.instr_ptr = 0;
    }

    pub fn init(&mut self, revargs: Vec<String>) {
        // synthetic call context
        // - RetVal
        // - # input redirs
        // - InvocationTarget
        // - Cwd
        // - Args + argn
        let argc = revargs.len();
        self.allocate(6 + argc);
        self.push_null(); // retval allocation
        self.push_val(0); // # env var settings
        self.push_val(0); // # input redirections
        self.push_null(); // invocation target
        self.push_null(); // cwd
        for arg in revargs {
            self.push_val(value::DynString(arg));
        }
        self.push_val(argc); // nargs

        self.instr_ptr = usize::max_value();
        self.prepare_call();
        self.instr_ptr = 0;
    }

    /// The current instruction address pointer
    pub fn instr_addr(&self) -> usize {
        self.instr_ptr
    }

    pub fn stackp(&self) -> usize {
        self.stack_ptr
    }
    fn stackp_next(&mut self) -> usize {
        let stackp = self.stack_ptr;
        self.stack_ptr += 1;
        ltrace!("stackp++ {}", stackp);
        stackp
    }

    pub fn frame_addr(&self, offset: usize) -> usize {
        self.frame_ptr + offset
    }

    pub fn arg_addr(&self, argn: usize) -> Result<usize> {
        // -n for stack-frame info: retaddr etc
        // -1 because fp points 1 beyond last
        let &Self { frame_ptr, .. } = self;
        let offset = 1 + self.call_stack_data().len() + argn;
        if offset > frame_ptr {
            temg!(
                "Arg offset too big: fp:{fp} - 1 - stack_data:{sd} - argn:{an}",
                fp = frame_ptr,
                sd = self.call_stack_data().len(),
                an = argn
            )
        }
        Ok(frame_ptr - offset)
    }

    fn call_stack_data(&self) -> [usize; 2] {
        [self.frame_ptr, self.instr_ptr]
    }
    pub fn ret_instr_addr(&self) -> Result<usize> {
        let &Self { frame_ptr, .. } = self;
        if frame_ptr < 1 {
            return temg!("cannot read below frame {}", frame_ptr);
        }
        Ok(*te!(
            self.stack_get(self.frame_ptr - 1),
            "frameptr {}",
            frame_ptr
        ))
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
    pub fn nargs(&self) -> Result<usize> {
        let vm = self;

        let &nargs: &usize = te!(vm.arg_get(0));
        Ok(nargs)
    }
    pub fn number_inputs(&self) -> Result<usize> {
        Ok(*te!(self.arg_get(te!(self.nargs()) + 3)))
    }
    pub fn number_environments(&self) -> Result<usize> {
        Ok(*te!(self.arg_get(
            te!(self.nargs()) + 3 + te!(self.number_inputs()) + 1
        )))
    }
    pub fn call_target_func_addr(&self) -> Result<usize> {
        let vm = self;

        let nargs = te!(vm.nargs());
        let nargs = te!(vm.arg_get_val(nargs + 2));
        let &value::FuncAddr(addr) = te!(nargs.try_ref());
        Ok(addr)
    }
    pub fn ret_cell_addr(&self) -> Result<usize> {
        let vm = self;

        let nargs = te!(vm.nargs());
        let ninps = te!(vm.number_inputs());
        let nenvs = te!(vm.number_environments());
        vm.arg_addr(nargs + 3 + ninps + 1 + nenvs + 1)
    }
    pub fn ret_cell_mut(&mut self) -> Result<&mut Value> {
        let vm = self;
        let addr = te!(vm.ret_cell_addr());
        Ok(vm.stack_get_val_mut(addr))
    }
    pub fn set_ret_val_from_local(&mut self, fp_off: usize) -> Result<()> {
        let vm = self;

        let retval_src: Value = mem::take(vm.frame_get_val_mut(fp_off));
        ltrace!(
            "return local {:?} to {}",
            retval_src,
            te!(vm.ret_cell_addr())
        );
        *te!(vm.ret_cell_mut()) = retval_src;

        Ok(())
    }
    pub fn set_ret_val<V: Into<Value>>(&mut self, val: V) -> Result<()> {
        let vm = self;

        let retval_src = val.into();
        ltrace!("return val {:?} to {}", retval_src, te!(vm.ret_cell_addr()));
        *te!(vm.ret_cell_mut()) = retval_src;

        Ok(())
    }
    pub fn return_from_call(&mut self, frame_size: usize) -> Result<()> {
        let vm = self;

        let ret_instr = te!(vm.ret_instr_addr());
        let ret_fp = vm.ret_fp_addr();
        ltrace!("return fp[{}] inst[{}]", ret_fp, ret_instr);

        vm.dealloc(vm.call_stack_data().len() + frame_size);
        vm.frame_ptr = ret_fp;

        vm.jump(ret_instr);
        Ok(())
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

    pub fn frame_get_val(&self, offset: usize) -> &Value {
        self.stack_get_val(self.frame_addr(offset))
    }
    pub fn frame_get_val_mut(&mut self, offset: usize) -> &mut Value {
        self.stack_get_val_mut(self.frame_addr(offset))
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

    pub fn arg_get_val(&self, argn: usize) -> Result<&Value> {
        Ok(self.stack_get_val(te!(self.arg_addr(argn))))
    }
    pub fn arg_get_val_mut(&mut self, argn: usize) -> Result<&mut Value> {
        Ok(self.stack_get_val_mut(te!(self.arg_addr(argn))))
    }
    pub fn arg_get<T>(&self, argn: usize) -> Result<&T>
    where
        for<'s> &'s T: TryFrom<&'s Value, Error = &'s Value> + ValueTypeInfo,
    {
        te!(self.arg_get_val(argn)).try_ref()
    }
    pub fn arg_get_mut<T>(&mut self, argn: usize) -> Result<&mut T>
    where
        for<'s> &'s mut T: TryFrom<&'s mut Value, Error = &'s mut Value> + ValueTypeInfo,
    {
        te!(self.arg_get_val_mut(argn)).try_mut()
    }

    pub fn push_val<T>(&mut self, src: T)
    where
        T: Into<Value>,
    {
        let addr = self.stackp_next();
        self.stack_set(addr, src);
    }

    pub fn push_null(&mut self) {
        self.push_val(());
    }

    pub fn push_lit_str(&mut self, strid: usize) {
        self.push_val(value::LitString(strid));
    }

    // Pushes an array of all arguments passed to this call
    pub fn push_args(&mut self) -> Result<()> {
        self.push_val(value::Array {
            ptr: te!(self.arg_addr(0)),
        });
        Ok(())
    }

    // Pushes local var #
    pub fn push_local(&mut self, fp_off: usize) {
        let val = self.frame_get_val(fp_off).clone();
        self.push_val(val);
    }

    /// Grow the stack by `size`
    pub fn allocate(&mut self, size: usize) {
        ltrace!("alloc: {}", size);
        self.stack
            .resize_with(self.stack.len() + size, <_>::default);
    }

    pub fn dealloc(&mut self, size: usize) {
        let Self {
            stack, stack_ptr, ..
        } = self;
        stack.truncate(stack.len() - size);
        *stack_ptr -= size;
        ltrace!(
            "dealloc: stackp [{}] - {} = {}",
            stack.len(),
            size,
            stack_ptr
        );
    }

    pub fn eval_icode(&mut self, icode: &ICode) -> Result<()> {
        self.load_icode(&icode)
            .and_then(|_| self.run_instructions(None, icode))
    }

    pub fn debug_icode(&mut self, icode: &ICode) -> Result<()> {
        let debugger = te!(Debugger::open());
        self.load_icode(&icode)
            .and_then(|_| self.run_instructions(Some(debugger), icode))
    }

    pub fn load_icode(&mut self, icode: &ICode) -> Result<()> {
        for (s, i) in &icode.strings {
            ltrace!("Load literal string {} {}", i.id, s);
            self.add_string(i.clone(), s.clone());
        }
        Ok(())
    }

    pub fn run_instructions(
        &mut self,
        mut debugger: Option<Debugger>,
        icode: &ICode,
    ) -> Result<()> {
        let vm = self;

        while vm.instr_ptr < icode.instructions.len() {
            let instruction = &icode.instructions[vm.instr_ptr];
            if let Some(debugger) = debugger.as_mut() {
                te!(debugger.run(vm, icode));
            }
            vm.instr_ptr += 1;
            let mut success = instruction.borrow().operate_on(vm);
            #[cfg(feature = "vm_stack_trace")]
            {
                if let Err(_) = &success {
                    te!(vm.write_to(Ok(std::io::stderr())))
                }
            }
            success = success; // for compile warning
            te!(success);
        }
        Ok(())
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

    pub fn write_to<O>(&self, o: io::Result<O>) -> Result<()>
    where
        O: io::Write,
    {
        let vm = self;
        let mut o = te!(o);

        use error::te_writeln as w;
        //w!(o, "=== BIN_PATH ===")?;
        //for path in &vm.bin_path {
        //    w!(o, "> {}", path)?;
        //}
        w!(o, "=== STRING TABLE ===");
        let mut i = 0;
        for string in &vm.string_table {
            w!(o, "[{:4}] {:?}", i, string);
            i += 1;
        }
        let mut fp = vm.frame_ptr;
        let mut sp = vm.stack_ptr;
        let len = vm.stack.len();
        w!(o, "=== STACK ===");
        w!(o, "fp({fp}) sp({sp}) len({l})", l = len, sp = sp, fp = fp);
        for i in 0..len {
            let i = len - 1 - i;

            let pref = if i < 6 || fp < 7 {
                "(sys)"
            } else {
                let nargs: usize = *te!(vm.stack[fp - 3].try_ref());
                let n_inp_redir: usize = *te!(vm.stack[fp - 3 - nargs - 3].try_ref());
                let nenvs: usize = *te!(vm.stack[fp - 3 - nargs - 3 - n_inp_redir - 1].try_ref());
                match i {
                    i if fp == i => "fp",
                    i if fp - 1 == i => "ret instr",
                    i if fp - 2 == i => "ret frame",
                    i if fp - 3 == i => "nargs",
                    i if fp - 3 - nargs <= i && fp - 3 > i => "arg",
                    i if fp - 3 - nargs - 1 == i => "cwd",
                    i if fp - 3 - nargs - 2 == i => "target",
                    i if fp - 3 - nargs - 3 == i => "n inp redr",
                    i if fp - 3 - nargs - 3 - n_inp_redir <= i && fp - 3 - nargs - 3 > i => {
                        "inp redr"
                    }
                    i if fp - 3 - nargs - 3 - n_inp_redir - 1 == i => "nenvs",
                    i if fp - 3 - nargs - 3 - n_inp_redir - 1 - nenvs <= i
                        && fp - 3 - nargs - 3 - n_inp_redir - 1 > i =>
                    {
                        "env set"
                    }
                    i if fp - 3 - nargs - 3 - n_inp_redir - 1 - nenvs - 1 == i => {
                        sp = *te!(vm.stack[fp - 1].try_ref());
                        fp = *te!(vm.stack[fp - 2].try_ref());
                        w!(o, "--- frame {} ---", fp);
                        "retval"
                    }
                    i if sp == i => "sp",
                    _ => "",
                }
            };
            let cell = &vm.stack[i];
            w!(o, "{:10} [{:4}] {:?}", pref, i, cell);
        }
        w!(o, "=== STATE ===");
        w!(o, "- frame pointer    : {}", vm.frame_ptr);
        w!(o, "- stack pointer    : {}", vm.stack_ptr);
        w!(o, "- instr pointer    : {}", vm.instr_ptr);
        Ok(())
    }

    pub fn get_string_id(&self, id: usize) -> Result<&str> {
        Ok(te!(self.string_table.get(id), "strid {}", id))
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

    pub fn cleanup<F, E>(&mut self, fp_off: usize, _cln_name: &str, cln: F) -> Result<()>
    where
        F: FnOnce(&mut Job) -> result::Result<(), E>,
        result::Result<(), E>: error::IntoResult<super::ErrorKind, ()>,
    {
        let vm = self;

        let val = vm.frame_get_val_mut(fp_off);
        match val {
            &mut Value::Job(value::Job(proc_id)) => {
                let job: &mut Job = te!(vm.job_table.get_mut(proc_id), "Proc {}", proc_id);
                te!(cln(job));
            }
            v @ (Value::FuncAddr(_) | Value::LitString(_) | Value::Null(_) | Value::Natural(_)) => {
                ltrace!("No cleanup: {:?}", v);
                // No cleanup
            }
            other => panic!("{:?}", other),
        }
        Ok(())
    }

    pub fn val_into_string(&self, val: Value) -> Result<String> {
        let vm = self;

        Ok(match val {
            Value::LitString(value::LitString(id)) => te!(vm.get_string_id(id)).to_owned(),
            Value::DynString(value::DynString(s)) => s,
            other => error::temg!("Not a string value: {:?}", other),
        })
    }
    pub fn val_as_str<'a>(&'a self, val: &'a Value) -> Result<&'a str> {
        let vm = self;

        Ok(match val {
            &Value::LitString(value::LitString(id)) => te!(vm.get_string_id(id)),
            Value::DynString(value::DynString(s)) => s.as_str(),
            other => error::temg!("Not a string value: {:?}", other),
        })
    }

    pub fn add_job<C>(&mut self, child: C) -> usize
    where
        C: Into<Job>,
    {
        let Self { job_table: t, .. } = self;
        let id = t.len();
        let job = child.into();

        t.push_back(job);

        id
    }

    pub fn get_job_mut(&mut self, jobid: usize) -> Result<&mut Job> {
        let job = self.job_table.get_mut(jobid);
        Ok(te!(job, "jobid {}", jobid))
    }

    pub fn get_job(&self, jobid: usize) -> Result<&Job> {
        let job = self.job_table.get(jobid);
        Ok(te!(job, "jobid {}", jobid))
    }
}
