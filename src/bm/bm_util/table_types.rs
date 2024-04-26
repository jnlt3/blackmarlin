use cozy_chess::{Piece, Square};

pub type Butterfly<T> = [[[T; Square::NUM]; Square::NUM]; 2];
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];
pub type Threats<T> = [T; 2];

pub fn new_butterfly_table<T: Copy>(default: T) -> Butterfly<T> {
    [[[default; Square::NUM]; Square::NUM]; 2]
}

pub fn new_piece_to_table<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::NUM]; Piece::NUM]
}

pub fn new_threats_table<T: Copy>(default: T) -> Threats<T> {
    [default; 2]
}
