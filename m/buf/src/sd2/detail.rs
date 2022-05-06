#[macro_export]
macro_rules! serializable {
    (
        $slf:ident
            $( <
               $( $lt:lifetime , )*
               $( $tp:ident $( : { $($tr:tt)+ } )?  , )*
               $( const $cn:ident : $cnt:ty , )*
               > )?
            ,
        $ser:expr,
        $dsr:expr
    ) => {
        impl
            $( < $($lt,)*
                $($tp $( : $($tr)+ )? ,)*
                $(const $cn : $cnt ,)* > )?
        $crate::sd2::WriteOut for $slf
            $( < $($lt,)* $($tp,)* $(const $cn : $cnt ,)* > )?
        {
            fn write_out<O: std::io::Write>(&self, dst: &mut O) -> std::io::Result<()> {
                let f: fn(&_, &mut _) -> _ = $ser;
                f(self, dst)
            }
        }

        impl
            $( < $($lt,)*
                $($tp $( : $($tr)+ )? ,)*
                $(const $cn : $cnt ,)* > )?
        $crate::sd2::ReadIn for $slf
            $( < $($lt,)* $($tp,)* $(const $cn : $cnt ,)* > )?
        {
            fn read_in<I: std::io::Read>(inp: &mut I) -> $crate::sd2::Result<Self> {
                let f: fn(&mut _) -> _ = $dsr;
                f(inp)
            }
        }
    };
}
#[macro_export]
macro_rules! sd {
    (
        $slf:ty,
        $ser:expr,
        $dsr:expr
    ) => {
        impl $crate::sd2::WriteOut for $slf {
            fn write_out<O: std::io::Write>(&self, dst: &mut O) -> std::io::Result<()> {
                type R<T> = std::io::Result<T>;

                let f: fn(&$slf, &mut O) -> R<()> = $ser;

                f(self, dst)
            }
        }

        impl $crate::sd2::ReadIn for $slf {
            fn read_in<I: std::io::Read>(inp: &mut I) -> $crate::sd2::Result<$slf> {
                type R<T> = $crate::sd2::Result<T>;

                let f: fn(&mut I) -> R<$slf> = $dsr;

                f(inp)
            }
        }
    };
}

#[macro_export]
macro_rules! sd_struct {
    (
        $slf:ident $(, $field:ident)*
    ) => {
        $crate::sd![
            $slf,
            |s: &$slf, d| {
                $(
                    s.$field.write_out(d)?;
                )*
                Ok(())
            },
            |inp|
                Ok(
                    Self {
                        $(
                            $field: error::te!($crate::sd2::ReadIn::read_in(inp)),
                        )*
                    }
                )
        ];
    };
}

#[macro_export]
macro_rules! sd_type {
    (
        $slf:ident $(, $field:ident, $fieldn:literal )*
    ) => {
        $crate::sd![
            $slf,
            |s, d| match s {
                $(
                    Self::$field(sub) => sub.write_with_code($fieldn, d),
                )*
            },
            |inp| Ok(match te!(u8::read_in(inp)) {
                $(
                    $fieldn => Self::$field(error::te!($crate::sd2::ReadIn::read_in(inp))),
                )*
                other => error::temg!("Invalid {}: {:?}", stringify!($slf), other),
            })
        ];
    };
}

#[macro_export]
macro_rules! sd_enum {
    (
        $slf:ident $(, $field:ident, $fieldn:literal )*
    ) => {
        $crate::sd![
            $slf,
            |s, d| match s {
                $(
                    Self::$field => <u8 as $crate::sd2::WriteOut>::write_out(&$fieldn, d),
                )*
            },
            |inp| Ok(match te!(u8::read_in(inp)) {
                $(
                    $fieldn => Self::$field,
                )*
                other => error::temg!("Invalid {}: {:?}", stringify!($slf), other),
            })
        ];
    };
}

#[macro_export]
macro_rules! sd_as {
    ($slf:ty, $ser:expr, $dsr:expr) => {
        $crate::sd! {
            $slf,
            |s: &$slf, d| $ser(s).write_out(d),
            |inp| Ok($dsr(te!(<_>::read_in(inp))))
        }
    };
}
macro_rules! as_bytes {
    ($slf:ty) => {
        $crate::sd_as! {
            $slf,
            |s: &$slf| s.to_be_bytes(),
            <$slf>::from_be_bytes
        }
    };
}

use super::*;
use error::te;

impl<A: WriteOut, B: WriteOut> WriteOut for (A, B) {
    fn write_out<O: io::Write>(&self, dst: &mut O) -> io::Result<()> {
        self.0.write_out(dst)?;
        self.1.write_out(dst)
    }
}
impl<A: ReadIn, B: ReadIn> ReadIn for (A, B) {
    fn read_in<I: io::Read>(inp: &mut I) -> Result<Self> {
        Ok((te!(<_>::read_in(inp)), te!(<_>::read_in(inp))))
    }
}

impl<const N: usize> WriteOut for [u8; N] {
    fn write_out<O: io::Write>(&self, dst: &mut O) -> io::Result<()> {
        dst.write_all(self.as_slice())
    }
}
impl<const N: usize> ReadIn for [u8; N] {
    fn read_in<I: io::Read>(inp: &mut I) -> Result<Self> {
        let mut bytes = [0; N];
        te!(inp.read_exact(bytes.as_mut_slice()));
        Ok(bytes)
    }
}

impl WriteOut for [u8] {
    fn write_out<O: io::Write>(&self, dst: &mut O) -> io::Result<()> {
        self.len().write_out(dst)?;
        dst.write_all(self)?;
        Ok(())
    }
}

impl<'t, T: WriteOut> WriteOut for &'t T {
    fn write_out<O: io::Write>(&self, dst: &mut O) -> io::Result<()> {
        (*self).write_out(dst)
    }
}

as_bytes!(u8);
as_bytes!(u16);
as_bytes!(usize);
sd_as![bool, |&b| if b { 0u8 } else { 1 }, |b: u8| if b == 0u8 {
    true
} else {
    false
}];
sd![String, |s, d| s.as_bytes().write_out(d), |inp| Ok(te!(
    String::from_utf8(te!(<_>::read_in(inp)))
))];

fn as_iter<C, O>(col: &C, dst: &mut O) -> io::Result<()>
where
    for<'r> &'r C: IntoIterator,
    for<'r> <&'r C as IntoIterator>::IntoIter: ExactSizeIterator,
    for<'r> <&'r C as IntoIterator>::Item: WriteOut,
    O: io::Write,
{
    let iter = col.into_iter();
    let len = iter.len();

    len.write_out(dst)?;

    for item in iter {
        item.write_out(dst)?;
    }

    Ok(())
}
fn from_iter<T, C, I>(inp: &mut I) -> Result<C>
where
    C: std::iter::FromIterator<T>,
    T: ReadIn,
    I: io::Read,
{
    let len = te!(usize::read_in(inp));

    let mut buf = Vec::new();
    buf.reserve(len);

    for _ in 0..len {
        buf.push(te!(<_>::read_in(inp)));
    }

    Ok(buf.into_iter().collect())
}

serializable! {
    Vec<T: { WriteOut + ReadIn },>,
    as_iter::<Vec<T>, _>,
    from_iter
}

use std::collections::HashMap;

serializable! {
    HashMap <
        K: { Eq + std::hash::Hash + WriteOut + ReadIn },
        V: { WriteOut + ReadIn },
    >,
    as_iter::<HashMap<K, V>, _>,
    from_iter
}

use std::collections::VecDeque;

serializable![
    VecDeque <
        T: { WriteOut + ReadIn },
    >,
    as_iter::<VecDeque<T>, _>,
    from_iter
];

serializable![
    Box<
        T: { WriteOut + ReadIn },
    >,
    |bx: &Box<T>, dst| bx.as_ref().write_out(dst),
    |inp| Ok(Box::new(te!(T::read_in(inp))))
];
