use cozy_chess::{BitBoard, Board, Color, Piece};

const WHITE: BitBoard = BitBoard(0xAA55AA55AA55AA55);

pub fn is_ocb(board: &Board) -> bool {
    let kings = board.pieces(Piece::King);
    let pawns = board.pieces(Piece::Pawn);
    let bishops = board.pieces(Piece::Bishop);
    if board.occupied() == (kings | pawns | bishops) {
        let w_bishops = board.colors(Color::White) & bishops;
        let b_bishops = board.colors(Color::Black) & bishops;
        if !(w_bishops.is_empty() || b_bishops.is_empty()) {
            return ((w_bishops & WHITE).is_empty() && (b_bishops & !WHITE).is_empty())
                || ((w_bishops & !WHITE).is_empty() && (b_bishops & WHITE).is_empty());
        }
    }
    false
}
