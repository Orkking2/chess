use std::fmt;

/// Sometimes, bad stuff happens.
#[derive(Clone, Debug)]
#[cfg(feature = "std")]
pub enum Error {
    /// The FEN string is invalid
    InvalidFen { fen: String },

    /// The board created from BoardBuilder was found to be invalid
    InvalidBoard,

    /// An attempt was made to create a square from an invalid string
    InvalidSquare,

    /// An attempt was made to create a move from an invalid SAN string
    InvalidSanMove,

    /// An atempt was made to create a move from an invalid UCI string
    InvalidUciMove,

    /// An attempt was made to convert a string not equal to "1"-"8" to a rank
    InvalidRank,

    /// An attempt was made to convert a string not equal to "a"-"h" to a file
    InvalidFile,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFen{ fen: s } => write!(f, "Invalid FEN string: {}", s),
            Self::InvalidBoard => write!(f, "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"),
            Self::InvalidSquare => write!(f, "The string specified does not contain a valid algebraic notation square."),
            Self::InvalidSanMove => write!(f, "The string specified does not contain a valid SAN notation move"),
            Self::InvalidUciMove => write!(f, "The string specified does not contain a valid UCI notation move"),
            Self::InvalidRank => write!(f, "The string specified does not contain a valid rank."),
            Self::InvalidFile => write!(f, "The string specified does not contain a valid file.")
        }
    }
}

#[derive(Clone, Debug)]
#[cfg(not(feature = "std"))]
pub enum Error {
    InvalidFen,
    InvalidBoard,
    InvalidSquare,
    InvalidSanMove,
    InvalidUciMove,
    InvalidRank,
    InvalidFile,
}
