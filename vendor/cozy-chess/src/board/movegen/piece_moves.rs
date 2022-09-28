use crate::*;

/// A compact structure representing multiple moves for a piece on the board.
/// Iterate it to unpack its moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceMoves {
    /// The [`Piece`] that is moved.
    pub piece: Piece,
    /// The square to move the piece from.
    pub from: Square,
    /// The possible destination squares.
    pub to: BitBoard
}

impl IntoIterator for PieceMoves {
    type Item = Move;

    type IntoIter = PieceMovesIter;

    fn into_iter(self) -> Self::IntoIter {
        PieceMovesIter {
            moves: self,
            promotion: 0
        }
    }
}

impl PieceMoves {
    /// Get the number of [`Move`]s.
    pub fn len(&self) -> usize {
        const PROMOTION_MASK: BitBoard = BitBoard(
            Rank::First.bitboard().0 | Rank::Eighth.bitboard().0
        );
        let moves = if self.piece == Piece::Pawn {
            (self.to & !PROMOTION_MASK).len() +
            (self.to & PROMOTION_MASK).len() * 4
        } else {
            self.to.len()
        };
        moves as usize
    }

    /// Check if there are no [`Move`]s.
    pub fn is_empty(&self) -> bool {
        self.to.is_empty()
    }

    /// Check if it contains a given [`Move`].
    pub fn has(&self, mv: Move) -> bool {
        let has_promotion = mv.promotion.is_some();
        let is_promotion = self.piece == Piece::Pawn &&
            matches!(mv.to.rank(), Rank::First | Rank::Eighth);
        self.from == mv.from
            && self.to.has(mv.to)
            && (has_promotion == is_promotion)
    }
}

/// Iterator over the moves in a [`PieceMoves`] instance.
pub struct PieceMovesIter {
    moves: PieceMoves,
    promotion: u8
}

impl Iterator for PieceMovesIter {
    type Item = Move;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let from = self.moves.from;
        let to = self.moves.to.next_square()?;
        let is_promotion = self.moves.piece == Piece::Pawn &&
            matches!(to.rank(), Rank::First | Rank::Eighth);
        let promotion = if is_promotion {
            let promotion = match self.promotion {
                0 => Piece::Knight,
                1 => Piece::Bishop,
                2 => Piece::Rook,
                3 => Piece::Queen,
                _ => unreachable!()
            };
            if self.promotion < 3 {
                self.promotion += 1;
            } else {
                self.promotion = 0;
                self.moves.to ^= to.bitboard();
            }
            Some(promotion)
        } else {
            self.moves.to ^= to.bitboard();
            None
        };
        Some(Move {
            from,
            to,
            promotion
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for PieceMovesIter {
    fn len(&self) -> usize {
        self.moves.len() - self.promotion as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_handles_promotions() {
        let mv = PieceMoves {
            piece: Piece::Pawn,
            from: Square::A7,
            to: Square::A8.bitboard() | Square::B8.bitboard()
        };
        assert_eq!(mv.len(), 8);
        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 8);
        for len in (0..8).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }
    
    #[test]
    fn has_works() {
        let mv = PieceMoves {
            piece: Piece::King,
            from: Square::A7,
            to: get_king_moves(Square::A7)
        };
        assert!(!mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: Some(Piece::Queen)
        }));
        assert!(mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: None
        }));
    }

    #[test]
    fn has_handles_promotions() {
        let mv = PieceMoves {
            piece: Piece::Pawn,
            from: Square::A7,
            to: Square::A8.bitboard() | Square::B8.bitboard()
        };
        assert!(mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: Some(Piece::Queen)
        }));
        assert!(!mv.has(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: None
        }));
    }
}
