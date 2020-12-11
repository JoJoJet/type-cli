use std::error::Error as StdError;

pub use type_cli_derive::CLI;

mod args;
pub use args::{Argument, OptionalArg, Flag};

pub trait CLI : Sized {
    fn parse(args: impl std::iter::Iterator<Item=String>) -> Result<Self, Error>;
}

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

