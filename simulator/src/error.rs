use core::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    OutOfBounds,
    CellNotEmpty,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OutOfBounds => write!(f, "Out of bounds"),
            Error::CellNotEmpty => write!(f, "Cell is not empty"),
        }
    }
}
