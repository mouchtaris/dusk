pub const VERSION: &str = "0.0.1";

use std::io;

pub trait Show {
    fn write_to_impl<O>(&self, out: O) -> io::Result<()>
    where
        O: io::Write;
    fn write_to<O>(&self, out: io::Result<O>) -> io::Result<()>
    where
        O: io::Write,
    {
        out.and_then(|out| self.write_to_impl(out))
    }
}
