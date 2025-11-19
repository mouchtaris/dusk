use super::*;

pub(crate) struct StackView<'a> {
    pub fp: usize,
    pub sp: usize,
    pub stack: &'a [Value],
}

pub const FRAME_RET_INFO_LEN: usize = 2;
// Absolute addresses on the stack (index)
pub type Ptr = Result<usize>;
// Offset from frame-pointer (counting downwards: fp - off)
pub type Off = Result<usize>;
// An "address" data type, found as a value in a cell of the stack
pub type Addr = Result<usize>;
// A "length" data type, found as a value in a cell of the stack
pub type Len = Result<usize>;

impl<'a> StackView<'a> {
    // ------------------------------------------------------------------------
    // Reading stack section
    //
    pub fn get_val(&self, addr: usize) -> Result<&Value> {
        Ok(te!(self.stack.get(addr)))
    }
    pub fn get<T>(&self, addr: usize) -> Result<&T>
    where
        for<'r> &'r T: TryFrom<&'r Value, Error = &'r Value> + ValueTypeInfo,
    {
        te!(self.get_val(addr)).try_ref()
    }
    pub fn get_ptr<T>(&self, ptr: Ptr) -> Result<&T>
    where
        for<'r> &'r T: TryFrom<&'r Value, Error = &'r Value> + ValueTypeInfo,
    {
        Ok(te!(self.get(te!(ptr))))
    }
    pub fn get_usize(&self, ptr: Ptr) -> Result<usize> {
        Ok(te!(self.get_ptr(ptr).copied()))
    }

    // ------------------------------------------------------------------------
    // Core translations
    //
    pub fn fp_minus(&self, off: Off) -> Ptr {
        let &Self { fp, .. } = self;
        let n: usize = te!(off);
        if fp < n {
            return temg!("Cannot read below 0: fp:{fp} - {n}");
        }
        Ok(fp - n)
    }

    // ------------------------------------------------------------------------
    // Core stack objects
    //
    pub fn read_len(&self, off: Off) -> Len {
        let ptr: Ptr = self.fp_minus(off);
        Ok(te!(self.get_usize(ptr)))
    }
    pub fn read_addr(&self, off: Off) -> Addr {
        let ptr: Ptr = self.fp_minus(off);
        Ok(te!(self.get_usize(ptr)))
    }
    pub fn read_val(&self, off: Off) -> Result<&Value> {
        let ptr: Ptr = self.fp_minus(off);
        Ok(te!(self.get_val(te!(self.fp_minus(ptr)))))
    }
    pub fn read_array(&self, start: Off, end: Off) -> Result<&[Value]> {
        let Self { stack, .. } = self;
        Ok(&stack[(te!(start) + 1)..(te!(end) + 1)])
    }

    // ------------------------------------------------------------------------
    // Offsets section
    //
    pub fn ret_instr_off(&self) -> Off {
        Ok(1)
    }
    pub fn ret_fp_off(&self) -> Off {
        Ok(te!(self.ret_instr_off()) + 1)
    }
    pub fn args_off(&self) -> Off {
        Ok(te!(self.ret_fp_off()) + 1)
    }
    pub fn args_len(&self) -> Len {
        Ok(te!(self.read_len(self.args_off())))
    }
    pub fn args_last(&self) -> Off {
        Ok(te!(self.args_off()) + te!(self.args_len()))
    }
    pub fn cwd_off(&self) -> Off {
        Ok(te!(self.args_last()) + 1)
    }
    pub fn target_off(&self) -> Off {
        Ok(te!(self.cwd_off()) + 1)
    }
    pub fn inps_off(&self) -> Off {
        Ok(te!(self.target_off()) + 1)
    }
    pub fn inps_len(&self) -> Len {
        Ok(te!(self.read_len(self.inps_off())))
    }
    pub fn inps_last(&self) -> Off {
        Ok(te!(self.inps_off()) + te!(self.inps_len()))
    }
    pub fn envs_off(&self) -> Off {
        Ok(te!(self.inps_last()) + 1)
    }
    pub fn envs_len(&self) -> Len {
        Ok(te!(self.read_len(self.envs_off())))
    }
    pub fn envs_last(&self) -> Off {
        Ok(te!(self.envs_off()) + 2 * te!(self.envs_len()))
    }

    // ------------------------------------------------------------------------
    // Values section
    //
    pub fn ret_instr_addr(&self) -> Addr {
        Ok(te!(self.read_addr(self.ret_instr_off())))
    }
    pub fn ret_fp_addr(&self) -> Addr {
        Ok(te!(self.read_addr(self.ret_fp_off())))
    }
    pub fn cwd(&self) -> Result<&Value> {
        Ok(te!(self.read_val(self.cwd_off())))
    }
    pub fn target(&self) -> Result<&Value> {
        Ok(te!(self.read_val(self.target_off())))
    }
    pub fn args(&self) -> Result<&[Value]> {
        Ok(te!(self.read_array(self.args_off(), self.args_len())))
    }
    pub fn inputs(&self) -> Result<&[Value]> {
        Ok(te!(self.read_array(self.inps_off(), self.inps_len())))
    }
    pub fn envs(&self) -> Result<&[Value]> {
        Ok(te!(self.read_array(self.envs_off(), self.envs_len())))
    }
}
