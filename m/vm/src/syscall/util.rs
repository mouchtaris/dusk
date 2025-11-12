use crate::*;

pub struct FramePtr(pub usize);

impl<S: BorrowMut<Vm>> ValuesVmExt for S {}

pub trait ValuesVmExt: BorrowMut<Vm> {
    fn add_tmp_value(&mut self, value: impl Into<Value>) -> Result<FramePtr> {
        let vm: &mut Vm = self.borrow_mut();

        vm.allocate(1);
        Ok(FramePtr(te!(vm.push_val(value))))
    }

    fn add_tmp_bytes(&mut self, cmd_name: &str, bytes: impl Into<Vec<u8>>) -> Result<FramePtr> {
        let vm: &mut Vm = self.borrow_mut();

        let job = job::Job::Buffer(job::Buffer::Bytes(
            std::process::Command::new(cmd_name),
            bytes.into(),
        ));

        let job_id: value::Job = vm.add_job(job).into();

        Ok(te!(self.add_tmp_value(job_id)))
    }

    /// Push the given icode as a byte-chunk on the stack and return the
    /// frame_ptr for it.
    fn add_tmp_icode(&mut self, icode: &ICode) -> Result<FramePtr> {
        let mut bytes: Vec<u8> = <_>::default();
        te!(icode.write_to(Ok(&mut bytes)));
        Ok(te!(self.add_tmp_bytes("<lib>", bytes)))
    }
}
