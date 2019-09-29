use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Syntax(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Syntax(e)
    }
}

impl Error {
    pub fn at_byte(byte: usize) -> Self {
        Error::Syntax(format!(
            "Found a ] with no correspoding [ at byte: {}",
            byte
        ))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::Syntax(e) => write!(f, "{}", e),
        }
    }
}
