use cozy_chess::{Board, Color, Piece, Square};

const CORNER_BISHOP_FACTOR: i16 = 50;

pub fn frc_corner_bishop(board: &Board) -> i16 {
    let mut score = 0;

    let white = board.colors(Color::White);
    let black = board.colors(Color::Black);

    let w_pawns = board.pieces(Piece::Pawn) & white;
    let b_pawns = board.pieces(Piece::Pawn) & black;
    let w_bishops = board.pieces(Piece::Bishop) & white;
    let b_bishops = board.pieces(Piece::Bishop) & black;

    if w_bishops.has(Square::A1) && w_pawns.has(Square::B2) {
        score -= CORNER_BISHOP_FACTOR;
    }
    if w_bishops.has(Square::H1) && w_pawns.has(Square::G2) {
        score -= CORNER_BISHOP_FACTOR;
    }

    if b_bishops.has(Square::A8) && b_pawns.has(Square::B7) {
        score += CORNER_BISHOP_FACTOR;
    }
    if b_bishops.has(Square::H8) && b_pawns.has(Square::G7) {
        score += CORNER_BISHOP_FACTOR;
    }

    match board.side_to_move() {
        Color::White => score,
        Color::Black => -score,
    }
}
