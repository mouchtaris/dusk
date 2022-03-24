use super::{te, Result, TryFrom};

macro_rules! either {
    ($($t:tt)*) => {
        either::either![
            #[derive(Debug, Clone)]
            pub $($t)*
        ];
    }
}
macro_rules! name {
    ($($t:tt)*) => {
        either::name![
            #[derive(Debug, Clone)]
            pub $($t)*
        ];
    }
}

either![Value, Null, LitString, DynString, Natural, Array, Job];

name![Job = usize];
pub type Null = ();
pub type Natural = usize;
name![LitString = usize];
name![DynString = String];

#[derive(Debug, Clone)]
pub struct Array {
    pub ptr: usize,
}

impl Default for Value {
    fn default() -> Self {
        Self::Null(())
    }
}

impl Value {
    pub fn type_info_name(&self) -> &'static str {
        match self {
            Value::Null(_) => Null::type_info_name(),
            Value::LitString(_) => LitString::type_info_name(),
            Value::Natural(_) => Natural::type_info_name(),
            Value::Array(_) => Array::type_info_name(),
            Value::Job(_) => Job::type_info_name(),
            Value::DynString(_) => DynString::type_info_name(),
        }
    }

    pub fn try_into<T>(self) -> Result<T>
    where
        T: TryFrom<Self, Error = Self> + ValueTypeInfo,
    {
        match T::try_from(self) {
            Err(v) => wrong_type_error(v),
            Ok(t) => Ok(t),
        }
    }
    pub fn try_mut<'a, T>(&'a mut self) -> Result<&'a mut T>
    where
        &'a mut T: TryFrom<&'a mut Self, Error = &'a mut Value> + ValueTypeInfo,
    {
        match <&'a mut T>::try_from(self) {
            Err(v) => wrong_type_error(v),
            Ok(t) => Ok(t),
        }
    }
    pub fn try_ref<'a, T>(&'a self) -> Result<&'a T>
    where
        &'a T: TryFrom<&'a Self, Error = &'a Value> + ValueTypeInfo,
    {
        match <&'a T>::try_from(self) {
            Err(v) => wrong_type_error(v),
            Ok(t) => Ok(t),
        }
    }
    //pub fn copied(&self) -> Value {
    //    use Value::*;
    //    match self {
    //        Null(v) => Null(v.clone()),
    //        LitString(v) => LitString(v.clone()),
    //        Natural(v) => Natural(v.clone()),
    //        Array(v) => Array(v.clone()),
    //        Job(v) => Process(v.clone()),
    //    }
    //}
}

fn wrong_type_error<T, V>(v: V) -> Result<T>
where
    T: ValueTypeInfo,
    V: RuntimeTypeInfo,
{
    let wanted = T::type_info_name();
    let actual = v.runtime_type_info_name();
    te!(Err(format!("Not a {} but a {}", wanted, actual)))
}

pub trait ValueTypeInfo {
    fn type_info_name() -> &'static str;
}
pub trait RuntimeTypeInfo {
    fn runtime_type_info_name(&self) -> &str;
}

impl<'a, T> RuntimeTypeInfo for &'a T
where
    T: RuntimeTypeInfo,
{
    fn runtime_type_info_name(&self) -> &str {
        T::runtime_type_info_name(*self)
    }
}
impl<'a, T> RuntimeTypeInfo for &'a mut T
where
    T: RuntimeTypeInfo,
{
    fn runtime_type_info_name(&self) -> &str {
        T::runtime_type_info_name(*self)
    }
}
impl RuntimeTypeInfo for Value {
    fn runtime_type_info_name(&self) -> &str {
        match self {
            Value::Null(_) => "null",
            Value::LitString(_) => "lit-string",
            Value::Natural(_) => "natural",
            Value::Array(_) => "array",
            Value::Job(_) => "job",
            Value::DynString(_) => "dyn-string",
        }
    }
}

impl<'a, T> ValueTypeInfo for &'a T
where
    T: ValueTypeInfo,
{
    fn type_info_name() -> &'static str {
        T::type_info_name()
    }
}
impl<'a, T> ValueTypeInfo for &'a mut T
where
    T: ValueTypeInfo,
{
    fn type_info_name() -> &'static str {
        T::type_info_name()
    }
}
impl ValueTypeInfo for LitString {
    fn type_info_name() -> &'static str {
        "LitString"
    }
}
impl ValueTypeInfo for Null {
    fn type_info_name() -> &'static str {
        "Null"
    }
}
impl ValueTypeInfo for Natural {
    fn type_info_name() -> &'static str {
        "Natural"
    }
}
impl<T> ValueTypeInfo for Option<T>
where
    T: ValueTypeInfo,
{
    fn type_info_name() -> &'static str {
        T::type_info_name()
    }
}
impl<T> ValueTypeInfo for Result<T>
where
    T: ValueTypeInfo,
{
    fn type_info_name() -> &'static str {
        T::type_info_name()
    }
}
impl ValueTypeInfo for Array {
    fn type_info_name() -> &'static str {
        "Array"
    }
}
impl ValueTypeInfo for Job {
    fn type_info_name() -> &'static str {
        "Job"
    }
}
impl ValueTypeInfo for DynString {
    fn type_info_name() -> &'static str {
        "DynString"
    }
}
