use core::fmt;

use crate::sim::Pos;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    OutOfBounds(Pos),
    Interseciton(Pos),
    MineEgress(Pos),
    DepositEgress(Pos),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OutOfBounds(pos) => write!(f, "{pos}: Out of bounds"),
            Error::Interseciton(pos) => write!(f, "{pos}: Cell is not empty"),
            Error::MineEgress(pos) => write!(f, "{pos}: Egresses of mines may only be connected to conveyors, combiners and factories"),
            Error::DepositEgress(pos) => write!(f, "{pos}: Only ingresses of mines may be connected to egresses of deposits"),
        }
    }
}
