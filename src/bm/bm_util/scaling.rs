use cozy_chess::{Board, Piece};

const BASE: i16 = 103;
const PAWNS: i16 = 1;
const KNIGHTS: i16 = 2;
const BISHOPS: i16 = 1;
const ROOKS: i16 = 6;
const QUEENS: i16 = 5;

pub fn scaling(board: &Board) -> i16 {
    let mut scaling = BASE;
    scaling += board.pieces(Piece::Pawn).popcnt() as i16 * PAWNS;
    scaling += board.pieces(Piece::Knight).popcnt() as i16 * KNIGHTS;
    scaling += board.pieces(Piece::Bishop).popcnt() as i16 * BISHOPS;
    scaling += board.pieces(Piece::Rook).popcnt() as i16 * ROOKS;
    scaling += board.pieces(Piece::Queen).popcnt() as i16 * QUEENS;
    scaling
}
