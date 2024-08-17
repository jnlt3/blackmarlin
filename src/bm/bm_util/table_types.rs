use cozy_chess::{Piece, Square};

pub type Butterfly<T> = [[T; Square::NUM]; Square::NUM];
pub type PieceSq<T> = [[T; Square::NUM]; Piece::NUM];

pub fn new_butterfly_table<T: Copy>(default: T) -> Butterfly<T> {
    [[default; Square::NUM]; Square::NUM]
}

pub fn new_piece_sq_table<T: Copy>(default: T) -> PieceSq<T> {
    [[default; Square::NUM]; Piece::NUM]
}
