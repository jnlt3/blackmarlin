use cozy_chess::{Piece, Square};

pub type Sq<T> = [T; Square::NUM];
pub type Butterfly<T> = Sq<Sq<T>>;
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];

pub fn new_sq_table<T: Copy>(default: T) -> Sq<T> {
    [default; Square::NUM]
}

pub fn new_butterfly_table<T: Copy>(default: T) -> Butterfly<T> {
    new_sq_table(new_sq_table(default))
}

pub fn new_piece_to_table<T: Copy>(default: T) -> PieceTo<T> {
    [new_sq_table(default); Piece::NUM]
}
