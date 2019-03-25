
use std::error;
use std::fmt;
use std::num::ParseIntError;
use std::convert::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    GenericError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "somethin went wrong!")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "somethin went wrong!"
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::GenericError
    }
}
