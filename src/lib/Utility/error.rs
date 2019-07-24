use std::fmt::{self, Display, Formatter};
use std::io;
use self::Error::*;

/// TODO : Add an Error Handling struct here.
#[derive(Clone,Debug,PartialEq)]
pub enum Error {
    Msg(String),
    Eof,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Msg(ref string) => write!(formatter, "{}", string),
            Eof => write!(formatter, "end of file"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Msg(error.to_string())
    }
}

impl<'a> From<&'a Error> for Error {
    fn from(error: &'a Error) -> Self {
        error.clone()
    }
}