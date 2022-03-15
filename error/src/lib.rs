pub const VERSION: &str = "0.0.1";

pub type FileTrace = &'static str;

pub type LineTrace = u32;

pub type Frame = (FileTrace, LineTrace);

pub type Trace = Vec<Frame>;

#[derive(Debug)]
pub struct Error<Kind> {
    pub kind: Kind,
    pub trace: Trace,
}

pub type Result<T, K> = std::result::Result<T, Error<K>>;

pub trait IntoResult<K, T> {
    fn into_result(self) -> Result<T, K>;
}

#[macro_export]
macro_rules! te {
    ($e:expr) => {
        $crate::IntoResult::into_result($e).map_err(|mut e| {
            e.trace.push((file!(), line!()));
            e
        })?
    };
}

#[macro_export]
macro_rules! error_kind {
    ($e:ident { $($n:ident = $t:ty)* }) => {
        #[derive(Debug)]
        pub enum $e {
            $( $n($t), )*
            None(())
        }

        $(
        impl <T> $crate::IntoResult<$e, T> for std::result::Result<T, $t> {
            fn into_result(self) -> $crate::Result<T, $e> {
                self.map_err(|e|
                    $crate::Error {
                        kind: $e::$n(e),
                        trace: <_>::default(),
                    }
                )
            }
        }
        )*

        impl <T> $crate::IntoResult<$e, T> for std::option::Option<T> {
            fn into_result(self) -> $crate::Result<T, $e> {
                self.ok_or(
                    $crate::Error {
                        kind: $e::None(()),
                        trace: <_>::default(),
                    }
                )
            }
        }
    }
}

impl<T, K> IntoResult<K, T> for std::result::Result<T, Error<K>> {
    fn into_result(self) -> Result<T, K> {
        self
    }
}
