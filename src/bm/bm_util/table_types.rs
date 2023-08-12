use cozy_chess::{Piece, Rank, Square};

pub type Butterfly<T> = [[T; Square::NUM]; Square::NUM];
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];
pub type PawnCompressed<T> = [[T; Rank::NUM]; Piece::NUM + 1];

pub fn new_butterfly_table<T: Copy>(default: T) -> Butterfly<T> {
    [[default; Square::NUM]; Square::NUM]
}

pub fn new_piece_to_table<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::NUM]; Piece::NUM]
}

pub fn new_rank_to_table<T: Copy>(default: T) -> PawnCompressed<T> {
    [[default; Rank::NUM]; Piece::NUM + 1]
}
