use crate::error::InvalidError;
use std::str::FromStr;

/// Describe a rank (row) on a chess board
#[repr(u8)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum Rank {
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
    Sixth = 5,
    Seventh = 6,
    Eighth = 7,
}

/// How many ranks are there?
pub const NUM_RANKS: usize = 8;

/// Enumerate all ranks
pub const ALL_RANKS: [Rank; NUM_RANKS] = [
    Rank::First,
    Rank::Second,
    Rank::Third,
    Rank::Fourth,
    Rank::Fifth,
    Rank::Sixth,
    Rank::Seventh,
    Rank::Eighth,
];

impl Rank {
    /// Convert a `usize` into a `Rank` (the inverse of into_index).  If the number is > 7, wrap
    /// around.
    #[inline(always)]
    pub const fn from_index(i: usize) -> Rank {
        // match is optimized to no-op with opt-level=1 with rustc 1.53.0
        match i & 7 {
            0 => Rank::First,
            1 => Rank::Second,
            2 => Rank::Third,
            3 => Rank::Fourth,
            4 => Rank::Fifth,
            5 => Rank::Sixth,
            6 => Rank::Seventh,
            7 => Rank::Eighth,
            _ => unreachable!(),
        }
    }

    /// Go one rank down.  If impossible, wrap around.
    #[inline(always)]
    pub const fn down(&self) -> Rank {
        Rank::from_index(self.into_index().wrapping_sub(1))
    }

    /// Go one rank up.  If impossible, wrap around.
    #[inline(always)]
    pub const fn up(&self) -> Rank {
        Rank::from_index(self.into_index() + 1)
    }

    /// Convert this `Rank` into a `usize` between 0 and 7 (inclusive).
    #[inline(always)]
    pub const fn into_index(self) -> usize {
        self as usize
    }
}

impl FromStr for Rank {
    type Err = InvalidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 1 {
            return Err(InvalidError::Rank);
        }
        match s.chars().next().unwrap() {
            '1' => Ok(Rank::First),
            '2' => Ok(Rank::Second),
            '3' => Ok(Rank::Third),
            '4' => Ok(Rank::Fourth),
            '5' => Ok(Rank::Fifth),
            '6' => Ok(Rank::Sixth),
            '7' => Ok(Rank::Seventh),
            '8' => Ok(Rank::Eighth),
            _ => Err(InvalidError::Rank),
        }
    }
}
