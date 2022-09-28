use crate::{Square, File, Rank};

use core::ops::*;

/// A [bitboard](https://www.chessprogramming.org/Bitboards).
/// A bitboard is an ordered set of squares.
/// 
/// Operators are overloaded to work as set operations accordingly:
/// ```
/// # use cozy_chess_types::*;
/// let a1 = Square::A1.bitboard();
/// let b1 = Square::B1.bitboard();
/// let c1 = Square::C1.bitboard();
/// let x = a1 | b1;
/// let y = a1 | c1;
/// // Union
/// assert_eq!(x | y, a1 | b1 | c1);
/// // Intersection
/// assert_eq!(x & y, a1);
/// // Symmetric difference
/// assert_eq!(x ^ y, b1 | c1);
/// // Difference
/// assert_eq!(x - y, b1);
/// // Complement
/// assert_eq!(!x, BitBoard::FULL - x);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(
    /// The backing [`u64`]. A square is present in the set if the bit at `1 << square as u8` is set.
    pub u64
);

macro_rules! impl_math_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for BitBoard {
            type Output = Self;

            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self($trait::$fn(self.0, rhs.0))
            }
        }
    )*};
}
impl_math_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

macro_rules! impl_math_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for BitBoard {
            #[inline(always)]
            fn $fn(&mut self, rhs: Self) {
                $trait::$fn(&mut self.0, rhs.0)
            }
        }
    )*};
}
impl_math_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

impl Sub for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

impl SubAssign for BitBoard {

    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

macro_rules! impl_convert {
    ($($type:ty),*) => {$(
        impl From<$type> for BitBoard {
            fn from(value: $type) -> Self {
                value.bitboard()
            }
        }
    )*};
}
impl_convert!(File, Rank, Square);

// Rustdoc currently has a bug where it attempts to guess how to display a constant for some reason.
// This has the amazing effect of expanding the `bitboard!` macro's implementation,
// making the docs completely unreadable. This is why constants defined with `bitboard!` use two constants.
// Relevant issues:
// https://github.com/rust-lang/rust/issues/99630
// https://github.com/rust-lang/rust/issues/98929

impl BitBoard {
    /// An empty bitboard.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::EMPTY, bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    pub const EMPTY: Self = Self(0);

    /// A bitboard with every square.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::FULL, bitboard! {
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     X X X X X X X X
    /// });
    /// ```
    pub const FULL: Self = Self(!0);

    /// The edges on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::EDGES, bitboard! {
    ///     X X X X X X X X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X X X X X X X X
    /// });
    /// ```
    pub const EDGES: Self = Self::__EDGES;
    const __EDGES: Self = bitboard! {
        X X X X X X X X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X X X X X X X X
    };

    /// The corners of the board.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::CORNERS, bitboard! {
    ///     X . . . . . . X
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X . . . . . . X
    /// });
    /// ```
    pub const CORNERS: Self = Self::__CORNERS;
    const __CORNERS: Self = bitboard! {
        X . . . . . . X
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        X . . . . . . X
    };

    /// The dark squares on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::DARK_SQUARES, bitboard! {
    ///     . X . X . X . X
    ///     X . X . X . X .
    ///     . X . X . X . X
    ///     X . X . X . X .
    ///     . X . X . X . X
    ///     X . X . X . X .
    ///     . X . X . X . X
    ///     X . X . X . X .
    /// });
    /// ```
    pub const DARK_SQUARES: Self = Self::__DARK_SQUARES;
    const __DARK_SQUARES: Self = bitboard! {
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
    };

    /// The light squares on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::LIGHT_SQUARES, bitboard! {
    ///     X . X . X . X .
    ///     . X . X . X . X
    ///     X . X . X . X .
    ///     . X . X . X . X
    ///     X . X . X . X .
    ///     . X . X . X . X
    ///     X . X . X . X .
    ///     . X . X . X . X
    /// });
    /// ```
    pub const LIGHT_SQUARES: Self = Self::__LIGHT_SQUARES;
    const __LIGHT_SQUARES: Self = bitboard! {
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
        X . X . X . X .
        . X . X . X . X
    };

    /// Sus.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::AMOGUS, bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    #[doc(hidden)]
    pub const AMOGUS: Self = bitboard! {
        . . . . . . . .
        . . . . . . . .
        . . X X X . . .
        . . X . X X . .
        . . X X X X . .
        . . X . X . . .
        . . . . . . . .
        . . . . . . . .
    };
    
    /// Flip the bitboard's ranks.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert_eq!(bb.flip_ranks(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X . X . . .
    ///     . . X X X X . .
    ///     . . X . X X . .
    ///     . . X X X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn flip_ranks(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Flip the bitboard's files.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert_eq!(bb.flip_files(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . X X X . .
    ///     . . X X . X . .
    ///     . . X X X X . .
    ///     . . . X . X . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn flip_files(self) -> Self {
        // https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Horizontal
        const K1: u64 = 0x5555555555555555;
        const K2: u64 = 0x3333333333333333;
        const K4: u64 = 0x0F0F0F0F0F0F0F0F;
        let mut new = self.0;
        new = ((new >> 1) & K1) | ((new & K1) << 1);
        new = ((new >> 2) & K2) | ((new & K2) << 2);
        new = ((new >> 4) & K4) | ((new & K4) << 4);
        Self(new)
    }

    /// Count the number of squares in the bitboard.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::EMPTY.len(), 0);
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert_eq!(bb.len(), 12);
    /// ```
    #[inline(always)]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Check if a [`Square`] is set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert!(bb.has(Square::C3));
    /// assert!(!bb.has(Square::B2));
    /// ```
    #[inline(always)]
    pub const fn has(self, square: Square) -> bool {
        !self.is_disjoint(square.bitboard())
    }

    /// Check if a bitboard contains no squares in common with another.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb_a = bitboard! {
    ///     X X X . . . . .
    ///     X . X X . . . .
    ///     X X X X . . . .
    ///     X . X . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// let bb_b = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . X X X .
    ///     . . . . X . X X
    ///     . . . . X X X X
    ///     . . . . X . X .
    /// };
    /// assert!(bb_a.is_disjoint(bb_b));
    /// ```
    #[inline(always)]
    pub const fn is_disjoint(self, other: BitBoard) -> bool {
        self.0 & other.0 == Self::EMPTY.0
    }

    /// Check if a bitboard is a subset of another.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . X X X X X . .
    ///     . X X X X X X .
    ///     . X X . X X X .
    ///     . X X X X X X .
    ///     . X X X X X . .
    ///     . X X . X X . .
    ///     . . . . . . . .
    /// };
    /// let subset = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert!(subset.is_subset(bb));
    /// ```
    #[inline(always)]
    pub const fn is_subset(self, other: BitBoard) -> bool {
        other.0 & self.0 == self.0
    }

    /// Check if a bitboard is a superset of another.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// let superset = bitboard! {
    ///     . . . . . . . .
    ///     . X X X X X . .
    ///     . X X X X X X .
    ///     . X X . X X X .
    ///     . X X X X X X .
    ///     . X X X X X . .
    ///     . X X . X X . .
    ///     . . . . . . . .
    /// };
    /// assert!(superset.is_superset(bb));
    /// ```
    #[inline(always)]
    pub const fn is_superset(self, other: BitBoard) -> bool {
        other.is_subset(self)
    }

    /// Checks if the [`BitBoard`] is empty.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert!(BitBoard::EMPTY.is_empty());
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert!(!bb.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    /// Grabs the first square if the bitboard is not empty.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert!(BitBoard::EMPTY.next_square().is_none());
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert_eq!(bb.next_square(), Some(Square::C3));
    /// ```
    #[inline(always)]
    pub const fn next_square(self) -> Option<Square> {
        Square::try_index(self.0.trailing_zeros() as usize)
    }

    /// Iterate the squares in the bitboard, ordered by square.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = BitBoard::FULL;
    /// let squares = &Square::ALL;
    /// for (s1, &s2) in bb.iter().zip(squares) {
    ///     assert_eq!(s1, s2);
    /// }
    /// ```
    #[inline(always)]
    pub fn iter(self) -> BitBoardIter {
        BitBoardIter(self)
    }

    /// Iterate all subsets of a bitboard.
    /// Subsets are produced in lexicographic order; Each subset is greater than the last.
    /// ```
    /// # use cozy_chess_types::*;
    /// let bb = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// for subset in bb.iter_subsets() {
    ///     assert!(subset.is_subset(bb));
    /// }
    /// ```
    #[inline(always)]
    pub fn iter_subsets(self) -> BitBoardSubsetIter {
        BitBoardSubsetIter {
            set: self,
            subset: Self::EMPTY,
            finished: false
        }
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;

    type IntoIter = BitBoardIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FromIterator<Square> for BitBoard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        iter.into_iter().fold(Self::EMPTY, |bb, sq| bb | sq.bitboard())
    }
}

/// An iterator over the squares of a bitboard.
/// 
/// This `struct` is created by [`BitBoard::iter`]. See its documentation for more.
pub struct BitBoardIter(BitBoard);

impl Iterator for BitBoardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let square = self.0.next_square();
        if let Some(square) = square {
            self.0 ^= square.bitboard();
        }
        square
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for BitBoardIter {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len() as usize
    }
}

/// An iterator over the subsets of a bitboard.
/// 
/// This `struct` is created by [`BitBoard::iter_subsets`]. See its documentation for more.
pub struct BitBoardSubsetIter {
    set: BitBoard,
    subset: BitBoard,
    finished: bool
}

impl Iterator for BitBoardSubsetIter {
    type Item = BitBoard;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = self.subset;
        // Carry-Rippler trick to enumerate all subsets of a set.
        // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
        self.subset.0 = self.subset.0.wrapping_sub(self.set.0) & self.set.0;
        self.finished = self.subset.is_empty();
        Some(current)
    }
}

/// [`BitBoard`] literal macro.
/// ```
/// # use cozy_chess_types::*;
/// let bb = bitboard! {
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     X X X . X X X X
///     . . . X . . . .
///     . . . X . . . .
/// };
/// assert_eq!(bb, File::D.bitboard() ^ Rank::Third.bitboard());
/// ```
#[macro_export]
macro_rules! bitboard {
    (
        $a8:tt $b8:tt $c8:tt $d8:tt $e8:tt $f8:tt $g8:tt $h8:tt
        $a7:tt $b7:tt $c7:tt $d7:tt $e7:tt $f7:tt $g7:tt $h7:tt
        $a6:tt $b6:tt $c6:tt $d6:tt $e6:tt $f6:tt $g6:tt $h6:tt
        $a5:tt $b5:tt $c5:tt $d5:tt $e5:tt $f5:tt $g5:tt $h5:tt
        $a4:tt $b4:tt $c4:tt $d4:tt $e4:tt $f4:tt $g4:tt $h4:tt
        $a3:tt $b3:tt $c3:tt $d3:tt $e3:tt $f3:tt $g3:tt $h3:tt
        $a2:tt $b2:tt $c2:tt $d2:tt $e2:tt $f2:tt $g2:tt $h2:tt
        $a1:tt $b1:tt $c1:tt $d1:tt $e1:tt $f1:tt $g1:tt $h1:tt
    ) => {
        $crate::bitboard! { @__inner
            $a1 $b1 $c1 $d1 $e1 $f1 $g1 $h1
            $a2 $b2 $c2 $d2 $e2 $f2 $g2 $h2
            $a3 $b3 $c3 $d3 $e3 $f3 $g3 $h3
            $a4 $b4 $c4 $d4 $e4 $f4 $g4 $h4
            $a5 $b5 $c5 $d5 $e5 $f5 $g5 $h5
            $a6 $b6 $c6 $d6 $e6 $f6 $g6 $h6
            $a7 $b7 $c7 $d7 $e7 $f7 $g7 $h7
            $a8 $b8 $c8 $d8 $e8 $f8 $g8 $h8
        }
    };
    (@__inner $($occupied:tt)*) => {{
        const BITBOARD: $crate::BitBoard = {
            let mut index = 0;
            let mut bitboard = $crate::BitBoard::EMPTY;
            $(
                if $crate::bitboard!(@__square $occupied) {
                    bitboard.0 |= 1 << index;
                }
                index += 1;
            )*
            let _ = index;
            bitboard
        };
        BITBOARD
    }};
    (@__square X) => { true };
    (@__square .) => { false };
    (@__square $token:tt) => {
        compile_error!(
            concat!(
                "Expected only `X` or `.` tokens, found `",
                stringify!($token),
                "`"
            )
        )
    };
    ($($token:tt)*) => {
        compile_error!("Expected 64 squares")
    };
}
pub use bitboard as bitboard;

impl core::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "bitboard! {{")?;
            for &rank in Rank::ALL.iter().rev() {
                write!(f, "\n   ")?;
                for &file in &File::ALL {
                    if self.has(Square::new(file, rank)) {
                        write!(f, " X")?;
                    } else {
                        write!(f, " .")?;
                    }
                }
            }
            write!(f, "\n}}")?;
            Ok(())
        } else {
            write!(f, "BitBoard({:#018X})", self.0)
        }
    }
}
