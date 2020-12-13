use std::{convert::AsRef, str::FromStr};
use super::{Error, ArgRef};
use std::error::Error as StdError;

pub trait Argument : Sized {
    fn parse(arg: impl AsRef<str>, arg: ArgRef) -> Result<Self, Error>;
}

impl<T: FromStr> Argument for T
where <T as FromStr>::Err : StdError + 'static
{
    fn parse(val: impl AsRef<str>, arg: ArgRef) -> Result<Self, Error> {
        let val = val.as_ref();
        T::from_str(val).map_err(|e| Error::Parse(arg, Box::new(e)))
    }
}


pub trait OptionalArg : Sized {
    fn parse(arg: impl AsRef<str>, arg: ArgRef) -> Result<Self, Error>;
    fn default() -> Self;

    fn map_parse(val: Option<impl AsRef<str>>, arg: ArgRef) -> Result<Self, Error> {
        match val {
            Some(val) => Self::parse(val, arg),
            None => Ok(Self::default())
        }
    }
}

impl<T: Argument> OptionalArg for Option<T> {
    fn parse(val: impl AsRef<str>, arg: ArgRef) -> Result<Self, Error> {
        Some(T::parse(val, arg)).transpose()
    }
    fn default() -> Self {
        None
    }
}


pub trait Flag : Default {
    fn increment(&mut self);
}

impl Flag for bool {
    fn increment(&mut self) {
        *self = true;
    }
}

impl Flag for Option<()> {
    fn increment(&mut self) {
        *self = Some(());
    }
}

macro_rules! int_flag {
    ($int: ty) => {
        impl $crate::Flag for $int {
            fn increment(&mut self){
                *self += 1;
            }
        }
    }
}

int_flag!(usize);
int_flag!(u8);
int_flag!(u16);
int_flag!(u32);
int_flag!(u64);
int_flag!(isize);
int_flag!(i8);
int_flag!(i16);
int_flag!(i32);
int_flag!(i64);