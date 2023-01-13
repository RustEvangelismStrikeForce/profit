use core::fmt;

use sim::{Id, Pos};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Sim(sim::Error),
    NoPath(Id, Pos, Pos),
    /// TODO: proper error
    NoSolution,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Sim(e) => write!(f, "{e}"),
            Error::NoPath(d_id, d_pos, f_pos) => write!(
                f,
                "No path found between deposit {d_id:?} at {d_pos} and factory at {f_pos}"
            ),
            Error::NoSolution => write!(f, "No solution"),
        }
    }
}

impl From<sim::Error> for Error {
    fn from(e: sim::Error) -> Self {
        Self::Sim(e)
    }
}
