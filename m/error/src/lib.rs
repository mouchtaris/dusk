pub const VERSION: &str = "0.0.1";

pub type FileTrace = &'static str;

pub type LineTrace = u32;

pub type Comments = Vec<String>;

pub type Frame = (FileTrace, LineTrace, Comments);

pub type Trace = Vec<Frame>;

pub struct Error<Kind> {
    pub kind: Kind,
    pub trace: Trace,
}

mod error_debug;

pub type Result<T, K> = std::result::Result<T, Error<K>>;

pub trait IntoResult<K, T> {
    fn into_result(self) -> Result<T, K>;
}

#[macro_export]
macro_rules! ltrace {
    ($($t:tt)*) => {
        ::log::trace!("[{}:{}] {}", file!(), line!(), format_args!($($t)*))
    };
}
#[macro_export]
macro_rules! ldebug {
    ($($t:tt)*) => {
        ::log::debug!("[{}:{}] {}", file!(), line!(), format_args!($($t)*))
    };
}
#[macro_export]
macro_rules! lwarn {
    ($($t:tt)*) => {
        ::log::warn!("[{}:{}] {}", file!(), line!(), format_args!($($t)*))
    };
}
#[macro_export]
macro_rules! soft_todo {
    () => {
        $crate::soft_todo!("")
    };
    ($fmt:literal $(, $arg:expr)*) => {
        $crate::lwarn!("todo: {}", format_args!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! te {
    ($e:expr , $fmt:literal $(, $fmtargs:expr)*) => {
        $crate::IntoResult::into_result($e).map_err(|mut e| {
            e.trace.push((file!(), line!(), vec!(
                format!($fmt $(, $fmtargs)*)
            )));
            e
        })?
    };
    ($e:expr) => {
        $crate::IntoResult::into_result($e).map_err(|mut e| {
            e.trace.push((file!(), line!(), vec!()));
            e
        })?
    };
}

#[macro_export]
macro_rules! terr {
    ($e:expr) => {
        $crate::te!(Err($e))
    };
}

#[macro_export]
macro_rules! temg {
    ($l:literal $($a:tt)*) => {
        $crate::te!(Err(format!($l $($a)*)))
    };
}

#[macro_export]
macro_rules! te_writeln {
    ($trg:expr, $l:literal $($a:tt)*) => {
        $crate::te!(writeln!($trg, $l $($a)*))
    };
}

#[macro_export]
macro_rules! Error {
    ($($n:ident = $t:ty)*) => {
        $crate::error_kind! {
            ErrorKind {
                $($n = $t)*
            }
        }
        pub type Error = $crate::Error<ErrorKind>;
        pub type Result<T> = $crate::Result<T, ErrorKind>;
    }
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

impl<T, K> IntoResult<K, T> for Result<T, K> {
    fn into_result(self) -> Self {
        self
    }
}

impl<K> Error<K> {
    pub fn with_comment<S: Into<String>>(mut self, msg: S) -> Self {
        self.comment(msg);
        self
    }
    pub fn comment<S>(&mut self, msg: S)
    where
        S: Into<String>,
    {
        if let Some((_, _, comments)) = self.trace.first_mut() {
            comments.push(msg.into())
        }
    }
}
