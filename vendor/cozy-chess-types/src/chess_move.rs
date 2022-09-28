use core::str::FromStr;

use crate::*;

/// A chess move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    /// The square to move the piece from.
    pub from: Square,
    /// The square to move the piece to.
    pub to: Square,
    /// The promotion piece, if it exists.
    pub promotion: Option<Piece>
}

crate::helpers::simple_error! {
    /// The value was not a valid [`Move`].
    pub struct MoveParseError = "The value was not a valid Move.";
}

impl FromStr for Move {
    type Err = MoveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(s: &str) -> Option<Move> {
            Some(Move {
                from: s.get(0..2)?.parse().ok()?,
                to: s.get(2..4)?.parse().ok()?,
                promotion: if let Some(promotion) = s.get(4..5) {
                    let promotion = promotion.parse().ok()?;
                    if matches!(promotion, Piece::King | Piece::Pawn) {
                        None
                    } else {
                        Some(promotion)
                    }
                } else {
                    None
                }
            })
        }
        parse(s).ok_or(MoveParseError)
    }
}

impl core::fmt::Display for Move {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion)?;
        }
        Ok(())
    }
}
