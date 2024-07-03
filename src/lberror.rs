use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum LBError {
    HyperError(hyper::Error),
    NoPeerError,
}

impl fmt::Display for LBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LBError::HyperError(e) => write!(f, "Hyper Error: \n {}", e),
            LBError::NoPeerError => write!(f, "NoServiceAvailable"),
        }
    }
}

impl Error for LBError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LBError::HyperError(e) => Some(e),
            LBError::NoPeerError => None,
        }
    }
}
