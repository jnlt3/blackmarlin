crate::helpers::simple_enum! {
    /// A chess piece.
    /// Pieces are ordered by approximate material value.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Piece {
        /// A pawn.
        Pawn,
        /// A knight.
        Knight,
        /// A bishop.
        Bishop,
        /// A rook.
        Rook,
        /// A queen.
        Queen,
        /// A king.
        King
    }
}

crate::helpers::enum_char_conv! {
    Piece, PieceParseError {
        Pawn = 'p',
        Knight = 'n',
        Bishop = 'b',
        Rook = 'r',
        Queen = 'q',
        King = 'k'
    }
}
