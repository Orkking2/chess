use crate::board::Board;
use crate::error::InvalidError;
use crate::file::File;
use crate::movegen::MoveGen;
use crate::piece::Piece;
use crate::rank::Rank;
use crate::square::Square;
#[cfg(test)]
use crate::{ALL_PIECES, ALL_SQUARES};

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Represent a ChessMove in memory
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct ChessMove {
    source: Square,
    dest: Square,
    promotion: Option<Piece>,
}

impl ChessMove {
    /// An invalid move, can make `Option<ChessMove>` more efficient. See `into_option`.
    pub const NULL_MOVE: Self = Self {
        source: Square::A1,
        dest: Square::A1,
        promotion: None,
    };

    /// Check if this is an invalid (null) move.
    #[inline]
    pub fn is_null_move(&self) -> bool {
        *self == Self::NULL_MOVE
    }

    /// Convert this `ChessMove` into a **larger** but more useful `Option<ChessMove>` if `self` is `ChessMove::NULL_MOVE`.
    ///
    /// Call `.from_option` to compress back into a `ChessMove`.
    #[inline]
    pub fn into_option(self) -> Option<Self> {
        if self.is_null_move() {
            None
        } else {
            Some(self)
        }
    }

    /// Convert an `Option<ChessMove>` into a *smaller* but potentially invalid `ChessMove`.
    ///
    /// Call `.into_option` to expand back into an `Option<ChessMove>`.
    #[inline]
    pub const fn from_option(value: Option<Self>) -> Self {
        if let Some(mov) = value {
            mov
        } else {
            Self::NULL_MOVE
        }
    }

    /// Create a new chess move, given a source `Square`, a destination `Square`, and an optional
    /// promotion `Piece`
    #[inline]
    pub const fn new(source: Square, dest: Square, promotion: Option<Piece>) -> Self {
        Self {
            source,
            dest,
            promotion,
        }
    }

    /// Get the source square (square the piece is currently on).
    #[inline]
    pub const fn get_source(&self) -> Square {
        self.source
    }

    /// Get the destination square (square the piece is going to).
    #[inline]
    pub const fn get_dest(&self) -> Square {
        self.dest
    }

    /// Get the promotion piece (maybe).
    #[inline]
    pub const fn get_promotion(&self) -> Option<Piece> {
        self.promotion
    }
    /// Convert a SAN (Standard Algebraic Notation) move into a `ChessMove`
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square};
    ///
    /// let board = Board::default();
    /// assert_eq!(
    ///     ChessMove::from_san(&board, "e4").expect("e4 is valid in the initial position"),
    ///     ChessMove::new(Square::E2, Square::E4, None)
    /// );
    /// ```
    pub fn from_san(board: &Board, move_text: &str) -> Result<Self, InvalidError> {
        // Castles first...
        if move_text == "O-O" || move_text == "O-O-O" {
            let rank = board.side_to_move().to_my_backrank();
            let source_file = File::E;
            let dest_file = if move_text == "O-O" { File::G } else { File::C };

            let m = Self::new(
                Square::make_square(rank, source_file),
                Square::make_square(rank, dest_file),
                None,
            );
            if MoveGen::new_legal(&board).any(|l| l == m) {
                return Ok(m);
            } else {
                return Err(InvalidError::SanMove);
            }
        }

        // forms of SAN moves
        // a4 (Pawn moves to a4)
        // exd4 (Pawn on e file takes on d4)
        // xd4 (Illegal, source file must be specified)
        // 1xd4 (Illegal, source file (not rank) must be specified)
        // Nc3 (Knight (or any piece) on *some square* to c3
        // Nb1c3 (Knight (or any piece) on b1 to c3
        // Nbc3 (Knight on b file to c3)
        // N1c3 (Knight on first rank to c3)
        // Nb1xc3 (Knight on b1 takes on c3)
        // Nbxc3 (Knight on b file takes on c3)
        // N1xc3 (Knight on first rank takes on c3)
        // Nc3+ (Knight moves to c3 with check)
        // Nc3# (Knight moves to c3 with checkmate)

        // Because I'm dumb, I'm wondering if a hash table of all possible moves would be stupid.
        // There are only 186624 possible moves in SAN notation.
        //
        // Would this even be faster?  Somehow I doubt it because caching, but maybe, I dunno...
        // This could take the form of a:
        // struct CheckOrCheckmate {
        //      Neither,
        //      Check,
        //      CheckMate,
        // }
        // struct FromSan {
        //      piece: Piece,
        //      source: Vec<Square>, // possible source squares
        //      // OR
        //      source_rank: Option<Rank>,
        //      source_file: Option<File>,
        //      dest: Square,
        //      takes: bool,
        //      check: CheckOrCheckmate
        // }
        //
        // This could be kept internally as well, and never tell the user about such an abomination
        //
        // I estimate this table would take around 2 MiB, but I had to approximate some things.  It
        // may be less

        // This can be described with the following format
        // [Optional Piece Specifier] ("" | "N" | "B" | "R" | "Q" | "K")
        // [Optional Source Specifier] ( "" | "a-h" | "1-8" | ("a-h" + "1-8"))
        // [Optional Takes Specifier] ("" | "x")
        // [Full Destination Square] ("a-h" + "0-8")
        // [Optional Promotion Specifier] ("" | "N" | "B" | "R" | "Q")
        // [Optional Check(mate) Specifier] ("" | "+" | "#")
        // [Optional En Passant Specifier] ("" | " e.p.")

        let error = InvalidError::SanMove;
        let mut cur_index: usize = 0;
        let moving_piece = match move_text
            .get(cur_index..(cur_index + 1))
            .ok_or(error.clone())?
        {
            "N" => {
                cur_index += 1;
                Piece::Knight
            }
            "B" => {
                cur_index += 1;
                Piece::Bishop
            }
            "Q" => {
                cur_index += 1;
                Piece::Queen
            }
            "R" => {
                cur_index += 1;
                Piece::Rook
            }
            "K" => {
                cur_index += 1;
                Piece::King
            }
            _ => Piece::Pawn,
        };

        let mut source_file = match move_text
            .get(cur_index..(cur_index + 1))
            .ok_or(error.clone())?
        {
            "a" => {
                cur_index += 1;
                Some(File::A)
            }
            "b" => {
                cur_index += 1;
                Some(File::B)
            }
            "c" => {
                cur_index += 1;
                Some(File::C)
            }
            "d" => {
                cur_index += 1;
                Some(File::D)
            }
            "e" => {
                cur_index += 1;
                Some(File::E)
            }
            "f" => {
                cur_index += 1;
                Some(File::F)
            }
            "g" => {
                cur_index += 1;
                Some(File::G)
            }
            "h" => {
                cur_index += 1;
                Some(File::H)
            }
            _ => None,
        };

        let mut source_rank = match move_text
            .get(cur_index..(cur_index + 1))
            .ok_or(error.clone())?
        {
            "1" => {
                cur_index += 1;
                Some(Rank::First)
            }
            "2" => {
                cur_index += 1;
                Some(Rank::Second)
            }
            "3" => {
                cur_index += 1;
                Some(Rank::Third)
            }
            "4" => {
                cur_index += 1;
                Some(Rank::Fourth)
            }
            "5" => {
                cur_index += 1;
                Some(Rank::Fifth)
            }
            "6" => {
                cur_index += 1;
                Some(Rank::Sixth)
            }
            "7" => {
                cur_index += 1;
                Some(Rank::Seventh)
            }
            "8" => {
                cur_index += 1;
                Some(Rank::Eighth)
            }
            _ => None,
        };

        let takes = if let Some("x") = move_text.get(cur_index..(cur_index + 1)) {
            cur_index += 1;
            true
        } else {
            false
        };

        let dest = if let Some(s) = move_text.get(cur_index..(cur_index + 2)) {
            if let Ok(q) = Square::from_str(s) {
                cur_index += 2;
                q
            } else {
                let sq = Square::make_square(
                    source_rank.ok_or(error.clone())?,
                    source_file.ok_or(error.clone())?,
                );
                source_rank = None;
                source_file = None;
                sq
            }
        } else {
            let sq = Square::make_square(
                    source_rank.ok_or(error.clone())?,
                    source_file.ok_or(error.clone())?,
            );
            source_rank = None;
            source_file = None;
            sq
        };

        let promotion = if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
            match s {
                "N" => {
                    cur_index += 1;
                    Some(Piece::Knight)
                }
                "B" => {
                    cur_index += 1;
                    Some(Piece::Bishop)
                }
                "R" => {
                    cur_index += 1;
                    Some(Piece::Rook)
                }
                "Q" => {
                    cur_index += 1;
                    Some(Piece::Queen)
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
            let _maybe_check_or_mate = match s {
                "+" => {
                    cur_index += 1;
                    Some(false)
                }
                "#" => {
                    cur_index += 1;
                    Some(true)
                }
                _ => None,
            };
        }

        let ep = if let Some(s) = move_text.get(cur_index..) {
            s == " e.p."
        } else {
            false
        };

        //if ep {
        //    cur_index += 5;
        //}

        // Ok, now we have all the data from the SAN move, in the following structures
        // moving_piece, source_rank, source_file, taks, dest, promotion, maybe_check_or_mate, and
        // ep

        let mut found_move: Option<Self> = None;
        for m in &mut MoveGen::new_legal(board) {
            // check that the move has the properties specified
            if board.piece_on(m.get_source()) != Some(moving_piece) {
                continue;
            }

            if let Some(rank) = source_rank {
                if m.get_source().get_rank() != rank {
                    continue;
                }
            }

            if let Some(file) = source_file {
                if m.get_source().get_file() != file {
                    continue;
                }
            }

            if m.get_dest() != dest {
                continue;
            }

            if m.get_promotion() != promotion {
                continue;
            }

            if found_move.is_some() {
                return Err(error);
            }

            let piece_exists = board.piece_on(m.get_dest()).is_some();

            // takes is complicated, because of e.p.
            if !takes && piece_exists {
                continue;
            }

            if !ep && takes && !piece_exists {
                continue;
            }

            found_move = Some(m);
        }

        found_move.ok_or(error.clone())
    }

    /// Encode this `ChessMove` into a `u16`.
    ///
    /// This will properly encode `Piece::Pawn` and `Piece::King` despite these being illegal promotions.
    pub fn encode(&self) -> u16 {
        let Self {
            source,
            dest,
            promotion,
        } = self;
        let mut acc = 0u16;
        acc |= (source.to_int() as u16) << 10;
        acc |= (dest.to_int() as u16) << 4;
        acc |= promotion
            .map(|promotion| promotion as u16 + 1)
            .unwrap_or(0);
        acc
    }

    /// Decode a `u16` into its representative `ChessMove`.
    /// 
    /// Will decode promotions to `Piece::Pawn` and `Piece::King` despite these being illegal promotions.
    pub fn decode(coded: u16) -> Self {
        const SRCE_MASK: u16 = 0b1111_1100_0000_0000; // << 10
        const DEST_MASK: u16 = 0b0000_0011_1111_0000; // << 4
        const PROM_MASK: u16 = 0b0000_0000_0000_1111;

        Self {
            source: Square::new(((coded & SRCE_MASK) >> 10) as u8),
            dest: Square::new(((coded & DEST_MASK) >> 4) as u8),
            promotion: match coded & PROM_MASK {
                1 => Some(Piece::Pawn),
                2 => Some(Piece::Knight),
                3 => Some(Piece::Bishop),
                4 => Some(Piece::Rook),
                5 => Some(Piece::Queen),
                6 => Some(Piece::King),
                _ => None,
            },
        }
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.promotion {
            Some(x) => write!(f, "{}{}{}", self.source, self.dest, x),
            None => write!(f, "{}{}", self.source, self.dest),
        }
    }
}

//? Why does this exist?
//? What does it even mean for a move to be "less" than another?
impl Ord for ChessMove {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.source != other.source {
            self.source.cmp(&other.source)
        } else if self.dest != other.dest {
            self.dest.cmp(&other.dest)
        } else if self.promotion != other.promotion {
            match self.promotion {
                None => Ordering::Less,
                Some(x) => match other.promotion {
                    None => Ordering::Greater,
                    Some(y) => x.cmp(&y),
                },
            }
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for ChessMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

/// Convert a UCI `String` to a move. If invalid, return `None`
/// ```
/// use chess::{ChessMove, Square, Piece};
/// use std::str::FromStr;
///
/// let mv = ChessMove::new(Square::E7, Square::E8, Some(Piece::Queen));
///
/// assert_eq!(ChessMove::from_str("e7e8q").expect("Valid Move"), mv);
/// ```
impl FromStr for ChessMove {
    type Err = InvalidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = Square::from_str(s.get(0..2).ok_or(InvalidError::UciMove)?)?;
        let dest = Square::from_str(s.get(2..4).ok_or(InvalidError::UciMove)?)?;

        let mut promo = None;
        if s.len() == 5 {
            promo = Some(match s.chars().last().ok_or(InvalidError::UciMove)? {
                'q' => Piece::Queen,
                'r' => Piece::Rook,
                'n' => Piece::Knight,
                'b' => Piece::Bishop,
                _ => return Err(InvalidError::UciMove),
            });
        }

        Ok(Self::new(source, dest, promo))
    }
}

#[test]
fn test_basic_moves() {
    let board = Board::default();
    assert_eq!(
        ChessMove::from_san(&board, "e4").expect("e4 is valid in the initial position"),
        ChessMove::new(Square::E2, Square::E4, None)
    );
}

#[test]
fn encoding_decoding() {
    for source in ALL_SQUARES {
        for dest in ALL_SQUARES {
            for promotion in ALL_PIECES.iter().copied().map(Some).chain([None]) {
                let mov = ChessMove::new(source, dest, promotion);
                let encoded = mov.encode();
                let decoded = ChessMove::decode(encoded);
                assert_eq!(mov, decoded, "Successfully decoded");
            }
        }
    }
}
