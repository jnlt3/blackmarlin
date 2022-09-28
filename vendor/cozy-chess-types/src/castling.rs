use crate::*;

/// Castling rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastleRights {
    /// The rook file for short castling.
    pub short: Option<File>,
    /// The rook file for long castling.
    pub long: Option<File>
}

impl CastleRights {
    /// Empty [`CastleRights`].
    /// # Examples
    /// ``​`
    /// # use cozy_chess_types::*;
    /// assert_eq!(CastleRights::EMPTY, CastleRights {
    ///    short: None,
    ///    long: None
    /// });
    /// ``​`
    pub const EMPTY: CastleRights = CastleRights {
        short: None,
        long: None
    };
}
