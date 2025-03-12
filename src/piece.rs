use crate::color::Color;
use std::fmt;

/// Represent a chess piece as a very simple enum
#[repr(u8)]
#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub enum Piece {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

/// How many piece types are there?
pub const NUM_PIECES: usize = 6;

/// An array representing each piece type, in order of ascending value.
pub const ALL_PIECES: [Piece; NUM_PIECES] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

/// How many ways can I promote?
pub const NUM_PROMOTION_PIECES: usize = 4;

/// What pieces can I promote to?
pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Queen, Piece::Knight, Piece::Rook, Piece::Bishop];

impl Piece {
    /// Convert the `Piece` to a `usize` for table lookups.
    #[inline]
    pub const fn into_index(self) -> usize {
        self as usize
    }

    /// Convert the `Piece` to its FEN `char`
    pub fn to_char(&self) -> char {
        match *self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }

    pub fn with_color(&self, color: Color) -> PieceWithColor {
        PieceWithColor {
            piece: *self,
            color,
        }
    }

    /// Convert a piece with a color to a string.  White pieces are uppercase, black pieces are
    /// lowercase.
    ///
    /// ```
    /// use chess::{Piece, Color};
    ///
    /// assert_eq!(Piece::King.to_string(Color::White), "K");
    /// assert_eq!(Piece::Knight.to_string(Color::Black), "n");
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    pub fn to_string(self, color: Color) -> String {
        let piece = format!("{}", self);
        if color == Color::White {
            piece.to_uppercase()
        } else {
            piece
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

pub struct PieceWithColor {
    piece: Piece,
    color: Color,
}

impl fmt::Display for PieceWithColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            if (&self.color).into() {
                self.piece.to_char().to_ascii_uppercase()
            } else {
                self.piece.to_char()
            }
        )
    }
}
