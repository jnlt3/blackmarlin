use crate::*;

crate::helpers::simple_enum! {
    /// A file on a chessboard.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum File {
        /// The A file.
        A,
        /// The B file.
        B,
        /// The C file.
        C,
        /// The D file.
        D,
        /// The E file.
        E,
        /// The F file.
        F,
        /// The G file.
        G,
        /// The H file.
        H
    }
}

crate::helpers::enum_char_conv! {
    File, FileParseError {
        A = 'a',
        B = 'b',
        C = 'c',
        D = 'd',
        E = 'e',
        F = 'f',
        G = 'g',
        H = 'h'
    }
}

impl File {
    /// Flip the file.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(File::A.flip(), File::H);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::index_const(Self::H as usize - self as usize)
    }

    /// Get a bitboard with all squares on this file set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(File::B.bitboard(), bitboard! {
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(u64::from_ne_bytes([
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001
        ]) << self as u8)
    }

    /// Get a bitboard with all squares on adjacent files set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(File::C.adjacent(), bitboard! {
    ///     . X . X . . . .
    ///     . X . X . . . .
    ///     . X . X . . . .
    ///     . X . X . . . .
    ///     . X . X . . . .
    ///     . X . X . . . .
    ///     . X . X . . . .
    ///     . X . X . . . .
    /// });
    /// assert_eq!(File::A.adjacent(), bitboard! {
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    /// });
    /// assert_eq!(File::H.adjacent(), bitboard! {
    ///     . . . . . . X .
    ///     . . . . . . X .
    ///     . . . . . . X .
    ///     . . . . . . X .
    ///     . . . . . . X .
    ///     . . . . . . X .
    ///     . . . . . . X .
    ///     . . . . . . X .
    /// });
    /// ```
    #[inline(always)]
    pub const fn adjacent(self) -> BitBoard {
        const TABLE: [BitBoard; File::NUM] = {
            let mut table = [BitBoard::EMPTY; File::NUM];
            let mut i = 0;
            while i < table.len() {
                if i > 0 {
                    table[i].0 |= File::index_const(i - 1)
                        .bitboard().0;
                }
                if i < (table.len() - 1) {
                    table[i].0 |= File::index_const(i + 1)
                        .bitboard().0;
                }
                i += 1;
            }
            table
        };
        TABLE[self as usize]
    }
}
