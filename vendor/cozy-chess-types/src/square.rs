use core::convert::TryInto;
use core::str::FromStr;

use crate::*;

macro_rules! define_square_with_docs {
    ($($square:ident),*) => {
        crate::helpers::simple_enum! {
            /// A square on a chessboard.
            /// Squares are ordered in rank-major order (A1, B1, C1, ... H8).
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub enum Square {
                $(
                    #[doc = concat!("The ", stringify!($square), " square.")]
                    $square
                ),*
            }
        }
    }
}
define_square_with_docs! {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

crate::helpers::simple_error! {
    /// The value was not a valid [`Square`].
    pub struct SquareParseError = "The value was not a valid Square.";
}

impl FromStr for Square {
    type Err = SquareParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let file = chars.next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError)?;
        let rank = chars.next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError)?;
        if chars.next().is_some() {
            return Err(SquareParseError);
        }
        Ok(Square::new(file, rank))
    }
}

impl core::fmt::Display for Square {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl Square {
    /// Make a square from a file and a rank.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::new(File::A, Rank::First), Square::A1);
    /// ```
    #[inline(always)]
    pub const fn new(file: File, rank: Rank) -> Self {
        Self::index_const(((rank as usize) << 3) | file as usize)
    }

    /// Get the file of this square.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::A1.file(), File::A);
    /// ```
    #[inline(always)]
    pub const fn file(self) -> File {
        File::index_const(self as usize & 0b000111)
    }

    /// Get the rank of this square.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::A1.rank(), Rank::First);
    /// ```
    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::index_const(self as usize >> 3)
    }

    /// Get a bitboard with this square set.
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::B2.bitboard(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . X . . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self as u8)
    }

    /// Offsets the square towards the top right.
    /// # Panics
    /// Panic if the offset would put the square out of bounds.
    /// See [`Square::try_offset`] for a non-panicking variant.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::A1.offset(1, 2), Square::B3);
    /// ```
    pub const fn offset(self, file_offset: i8, rank_offset: i8) -> Square {
        if let Some(sq) = self.try_offset(file_offset, rank_offset) {
            sq
        } else {
            panic!("Offset would put square out of bounds.")
        }
    }

    /// Non-panicking version of [`Square::offset`].
    /// # Errors
    /// See [`Square::offset`]'s panics.
    #[inline(always)]
    pub const fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        let file_index = self.file() as i8 + file_offset;
        let rank_index = self.rank() as i8 + rank_offset;
        if file_index < 0 || file_index >= 8 || rank_index < 0 || rank_index >= 8 {
            return None;
        }
        let file = File::index_const(file_index as usize);
        let rank = Rank::index_const(rank_index as usize);
        Some(Square::new(file, rank))
    }

    /// Flip the file of this square.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::A1.flip_file(), Square::H1);
    /// ```
    #[inline(always)]
    pub const fn flip_file(self) -> Self {
        Self::index_const(self as usize ^ 0b000111)
    }

    /// Flip the rank of this square.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Square::A1.flip_rank(), Square::A8);
    /// ```
    #[inline(always)]
    pub const fn flip_rank(self) -> Self {
        Self::index_const(self as usize ^ 0b111000)
    }
}
