use failure::Fail;

/// Sometimes, bad stuff happens.
#[derive(Clone, Debug, Fail)]
pub enum Error {
    /// The FEN string is invalid
    #[fail(display = "Invalid FEN string: {}", fen)]
    InvalidFen { fen: String },

    /// The board created from BoardBuilder was found to be invalid
    #[fail(
        display = "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"
    )]
    InvalidBoard,

    /// An attempt was made to create a square from an invalid string
    #[fail(display = "The string specified does not contain a valid algebraic notation square. {}", info)]
    InvalidSquare{
        info: InvalidInfo,
    },

    /// An attempt was made to create a move from an invalid SAN string
    #[fail(display = "The string specified does not contain a valid SAN notation move")]
    InvalidSanMove,

    /// An atempt was made to create a move from an invalid UCI string
    #[fail(display = "The string specified does not contain a valid UCI notation move")]
    InvalidUciMove,

    /// An attempt was made to convert a string not equal to "1"-"8" to a rank
    #[fail(display = "The string specified does not contain a valid rank. {}", info)]
    InvalidRank{
        info: InvalidInfo
    },

    /// An attempt was made to convert a string not equal to "a"-"h" to a file
    #[fail(display = "The string specified does not contain a valid file. {}", info)]
    InvalidFile {
        info: InvalidInfo
    },
}

#[derive(Clone, Debug, Fail)]
pub enum InvalidInfo {
    #[fail(display = "Input is too short (expected {} but recieved {})", expected, recieved)]
    InputStringTooShort {
        expected: usize,
        recieved: usize,
    },

    #[fail(display = "File char ({}) was not matched", recieved)]
    FileCharNotMatched {
        recieved: char
    },
    
    #[fail(display = "Rank char ({}) was not matched", recieved)]
    RankCharNotMatched {
        recieved: char,
    },
}