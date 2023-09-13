use cozy_chess::{Piece, Square};

pub type Butterfly<T> = [[T; Square::NUM]; Square::NUM];
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];
pub type Threat<T> = [[T; 2]; 2];

pub fn new_butterfly_table<T: Copy>(default: T) -> Butterfly<T> {
    [[default; Square::NUM]; Square::NUM]
}

pub fn new_piece_to_table<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::NUM]; Piece::NUM]
}

pub fn new_threat<T: Copy>(default: T) -> Threat<T> {
    [[default; 2]; 2]
}
