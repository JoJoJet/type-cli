use std::error::Error as StdError;

pub use type_cli_derive::CLI;

mod args;
pub use args::{Argument, Flag, OptionalArg};

pub trait CLI: Sized {
    ///
    /// Parses the arguments as a command-line interface of the current type,
    /// returning errors as a value for manul handling.
    ///
    /// If you don't need fine control over program flow, use `CLI::processs` instead.
    fn parse(args: impl std::iter::Iterator<Item = String>) -> Result<Parse<Self>, Error>;
    ///
    /// Parses `std::env::args()` as a command-line interface of the current type.
    ///
    /// If an error occurs while parsing, it will be send to stderr and the process will exit.
    /// If the user enters `--help` or `-h`, help info will be sent to stderr and the process will exit.
    ///
    /// If you want finer control over program flow, use `CLI::parse` instead.
    fn process() -> Self {
        match Self::parse(std::env::args()) {
            Ok(Parse::Success(val)) => val,
            Ok(Parse::Help(help)) => {
                eprintln!("{}", help);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(101);
            }
        }
    }
}

///
/// A result of successful command-line interface parsing.
///
/// This is either a data structure holding the arguments passed to the program,
/// or a string containing help info about the current command.
pub enum Parse<T: CLI> {
    Success(T),
    Help(HelpInfo),
}

pub struct HelpInfo(pub &'static str);

impl std::fmt::Display for HelpInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
    #[error("Unknown subcommand `{0}`")]
    UnknownSub(String),
    #[error("Error parsing {0}:\n{1}")]
    Parse(ArgRef, Box<dyn StdError>),
}

/// A way to refer to an argument in an error.
pub enum ArgRef {
    Positional(usize),
    Named(&'static str),
}
use std::fmt::{self, Display};
impl Display for ArgRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &ArgRef::Positional(index) => {
                write!(f, "positional argument `{}`", index)
            }
            &ArgRef::Named(name) => {
                write!(f, "argument `{}`", name)
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
