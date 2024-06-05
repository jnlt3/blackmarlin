use cozy_chess::{Piece, Square};

pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];

pub fn new_piece_to_table<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::NUM]; Piece::NUM]
}
