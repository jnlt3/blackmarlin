use crate::*;

crate::helpers::simple_enum! {
    /// A rank on a chessboard.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Rank {
        /// The first rank.
        First,
        /// The second rank.
        Second,
        /// The third rank.
        Third,
        /// The fourth rank.
        Fourth,
        /// The fifth rank.
        Fifth,
        /// The sixth rank.
        Sixth,
        /// The seventh rank.
        Seventh,
        /// The eighth rank.
        Eighth
    }
}

crate::helpers::enum_char_conv! {
    Rank, RankParseError {
        First = '1',
        Second = '2',
        Third = '3',
        Fourth = '4',
        Fifth = '5',
        Sixth = '6',
        Seventh = '7',
        Eighth = '8'
    }
}

impl Rank {
    /// Flip the rank.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Rank::First.flip(), Rank::Eighth);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::index_const(Self::Eighth as usize - self as usize)
    }

    /// Get a bitboard with all squares on this rank set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Rank::Second.bitboard(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(0b11111111 << (self as u8 * 8))
    }

    /// Get a rank relative to some color.
    /// This flips the rank if viewing from black's perspective.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Rank::First.relative_to(Color::White), Rank::First);
    /// assert_eq!(Rank::First.relative_to(Color::Black), Rank::Eighth);
    /// ```
    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        if let Color::White = color {
            self
        } else {
            self.flip()
        }
    }
}
