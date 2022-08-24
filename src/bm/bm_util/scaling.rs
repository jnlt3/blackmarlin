use cozy_chess::{Board, Piece};

const BASE: i16 = 100;
const PAWNS: i16 = 0;
const KNIGHTS: i16 = 0;
const BISHOPS: i16 = 0;
const ROOKS: i16 = 0;
const QUEENS: i16 = 0;

pub fn scaling(board: &Board) -> i16 {
    let mut scaling = BASE;
    scaling += board.pieces(Piece::Pawn).popcnt() as i16 * PAWNS;
    scaling += board.pieces(Piece::Knight).popcnt() as i16 * KNIGHTS;
    scaling += board.pieces(Piece::Bishop).popcnt() as i16 * BISHOPS;
    scaling += board.pieces(Piece::Rook).popcnt() as i16 * ROOKS;
    scaling += board.pieces(Piece::Queen).popcnt() as i16 * QUEENS;
    scaling
}
