pub use cbor_adapt::*;

use {super::Result, error::te, std::io};

#[cfg(feature = "serde_cbor")]
mod cbor_adapt {
    use super::*;
    pub type CborError = serde_cbor::Error;

    pub fn ser<T, B>(mut buffer: B, t: &T) -> Result<()>
    where
        B: io::Write,
        T: serde::Serialize,
    {
        Ok(te!(serde_cbor::to_writer(&mut buffer, t)))
    }

    pub fn deser<T, B>(buffer: B) -> Result<T>
    where
        B: io::Read,
        T: for<'r> serde::Deserialize<'r>,
    {
        Ok(te!(serde_cbor::from_reader(buffer)))
    }

    pub fn copy<T>(src: &T) -> Result<T>
    where
        T: serde::Serialize,
        T: for<'r> serde::Deserialize<'r>,
    {
        let mut buffer = Vec::<u8>::new();
        te!(ser(&mut buffer, src));
        Ok(te!(deser(buffer.as_slice())))
    }
}

#[cfg(not(feature = "serde_cbor"))]
mod cbor_adapt {
    use {super::*, compile::Compiler};

    #[derive(Debug)]
    pub struct CborError;

    pub type T = Compiler;

    pub fn ser<B>(mut buffer: B, t: &T) -> Result<()>
    where
        B: io::Write,
    {
        Ok(te!(t.write_out(&mut buffer)))
    }

    pub fn deser<B>(mut buffer: B) -> Result<T>
    where
        B: io::Read,
    {
        Ok(te!(Compiler::read_in(&mut buffer)))
    }

    pub fn copy(src: &T) -> Result<T> {
        let mut buffer = Vec::<u8>::new();
        te!(ser(&mut buffer, src));
        Ok(te!(deser(buffer.as_slice())))
    }
}
