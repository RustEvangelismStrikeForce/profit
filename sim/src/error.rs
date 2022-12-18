use core::fmt;

use crate::board::Pos;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    OutOfBounds(Pos),
    Interseciton(Pos),
    MineEgress(Pos),
    DepositEgress(Pos),
    MultipleIngresses(Pos),
    Io(IoError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OutOfBounds(pos) => write!(f, "{pos}: Out of bounds"),
            Error::Interseciton(pos) => write!(f, "{pos}: Cell is not empty"),
            Error::MineEgress(pos) => write!(f, "{pos}: Egresses of mines may only be connected to conveyors, combiners and factories"),
            Error::DepositEgress(pos) => write!(f, "{pos}: Only ingresses of mines may be connected to egresses of deposits"),
            Error::MultipleIngresses(pos) => write!(f, "{pos}: Egresses may only be connected to a single ingress"),
            Error::Io(IoError::UnknownDepositSubtype(t)) => write!(f, "Unknown deposit subtype '{t}'"),
            Error::Io(IoError::UnknownMineSubtype(t)) => write!(f, "Unknown mine subtype '{t}'"),
            Error::Io(IoError::UnknownConveyorSubtype(t)) => write!(f, "Unknown conveyor subtype '{t}'"),
            Error::Io(IoError::UnknownCombinerSubtype(t)) => write!(f, "Unknown combiner subtype '{t}'"),
            Error::Io(IoError::UnknownFactorySubtype(t)) => write!(f, "Unknown factory subtype '{t}'"),
            Error::Io(IoError::UnknownProductSubtype(t)) => write!(f, "Unknown product subtype '{t}'"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IoError {
    UnknownDepositSubtype(u8),
    UnknownMineSubtype(u8),
    UnknownConveyorSubtype(u8),
    UnknownCombinerSubtype(u8),
    UnknownFactorySubtype(u8),
    UnknownProductSubtype(u8),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::Io(e)
    }
}
