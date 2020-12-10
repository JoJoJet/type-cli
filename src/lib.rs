use std::{convert::AsRef, error::Error as StdError, str::FromStr};

pub use type_cli_derive::CLI;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Expected an argument named `{0}`")]
    ExpectedNamed(&'static str),
    #[error("Expected an argument at position `{0}`")]
    ExpectedPositional(usize),
    #[error("Expected a value after argument `{0}`")]
    ExpectedValue(&'static str),
    #[error("Unknown flag `{0}`")]
    UnknownFlag(String),
    #[error("Unexpected positional argument `{0}`")]
    ExtraArg(String),
    #[error("Error parsing string `{0}`")]
    Parse(String, Box<dyn StdError>)
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}


pub trait Argument : Sized {
    fn parse(arg: impl AsRef<str>) -> Result<Self, Error>;
}

impl<T: FromStr> Argument for T
where <T as FromStr>::Err : StdError + 'static
{
    fn parse(arg: impl AsRef<str>) -> Result<Self, Error> {
        let arg = arg.as_ref();
        T::from_str(arg).map_err(|e| Error::Parse(arg.to_string(), Box::new(e)))
    }
}


pub trait OptionalArg : Sized {
    fn parse(arg: impl AsRef<str>) -> Result<Self, Error>;
    fn default() -> Self;

    fn map_parse(arg: Option<impl AsRef<str>>) -> Result<Self, Error> {
        match arg {
            Some(arg) => Self::parse(arg),
            None => Ok(Self::default())
        }
    }
}

impl<T: Argument> OptionalArg for Option<T> {
    fn parse(arg: impl AsRef<str>) -> Result<Self, Error> {
        Some(T::parse(arg)).transpose()
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
                *self = *self + 1;
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