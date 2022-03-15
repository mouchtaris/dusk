pub const VERSION: &str = "0.0.1";

::error::error_kind! {
    ErrorKind {
        Io = std::io::Error
        EnvVar = std::env::VarError
    }
}

pub use ::error::te;

pub type Result<T> = ::error::Result<T, ErrorKind>;
