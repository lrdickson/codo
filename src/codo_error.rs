use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    ContainerEngineFailure
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

// user-facing output
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.message) 
    }
}

impl error::Error for Error {}

impl Error {
    pub fn new(kind: ErrorKind, message: &str) -> Error {
        Error {
            kind: kind,
            message: message.to_string(),
       }
    }
}
