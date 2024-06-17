use cozy_chess::{Piece, Square};

pub type Butterfly<T> = [[T; Square::NUM]; Square::NUM];
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];
pub type PieceToPiece<T> = PieceTo<[T; Piece::NUM]>;

pub fn new_butterfly_table<T: Copy>(default: T) -> Butterfly<T> {
    [[default; Square::NUM]; Square::NUM]
}

pub fn new_piece_to_table<T: Copy>(default: T) -> PieceTo<T> {
    [[default; Square::NUM]; Piece::NUM]
}


pub fn new_piece_to_piece<T: Copy>(default: T) -> PieceToPiece<T> {
    new_piece_to_table([default; Piece::NUM])
}
