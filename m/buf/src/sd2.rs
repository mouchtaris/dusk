use std::{io, string};

error::Error! {
    Io = io::Error
    Utf8 = string::FromUtf8Error
    Other = String
}

pub trait WriteOut {
    fn write_out<O: io::Write>(&self, dst: &mut O) -> io::Result<()>;

    fn write_with_code<C, O: io::Write>(&self, code: C, dst: &mut O) -> io::Result<()>
    where
        C: WriteOut,
    {
        code.write_out(dst)?;
        self.write_out(dst)
    }
}

pub trait ReadIn: Sized {
    fn read_in<I: io::Read>(inp: &mut I) -> Result<Self>;
}

mod detail;
