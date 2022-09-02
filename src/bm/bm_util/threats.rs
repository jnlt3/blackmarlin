use cozy_chess::{BitBoard, Board, Color, File, Piece};

pub fn threats(board: &Board) -> (BitBoard, BitBoard) {
    let occupied = board.occupied();
    let white = board.colors(Color::White);
    let black = board.colors(Color::Black);

    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);

    let minors = knights | bishops;
    let majors = rooks | queens;
    let pieces = minors | majors;

    let mut w_threats = pawn_threats(pawns & white, Color::White) & pieces & black;
    let mut b_threats = pawn_threats(pawns & black, Color::Black) & pieces & white;

    for major in majors & black & !w_threats {
        if !(cozy_chess::get_knight_moves(major) & knights & white).is_empty() {
            w_threats |= major.bitboard();
        } else if !(cozy_chess::get_bishop_moves(major, occupied) & bishops & white).is_empty() {
            w_threats |= major.bitboard();
        }
    }

    for queen in queens & black & !w_threats {
        if !(cozy_chess::get_rook_moves(queen, occupied) & rooks & white).is_empty() {
            w_threats |= queen.bitboard();
        }
    }

    for major in majors & white & !b_threats {
        if !(cozy_chess::get_knight_moves(major) & knights & black).is_empty() {
            b_threats |= major.bitboard();
        } else if !(cozy_chess::get_bishop_moves(major, occupied) & bishops & black).is_empty() {
            b_threats |= major.bitboard();
        }
    }

    for queen in queens & white & !b_threats {
        if !(cozy_chess::get_rook_moves(queen, occupied) & rooks & black).is_empty() {
            b_threats |= queen.bitboard();
        }
    }
    (w_threats, b_threats)
}

fn pawn_threats(pawns: BitBoard, color: Color) -> BitBoard {
    match color {
        Color::White => ((pawns & !File::A.bitboard()) << 7) | ((pawns & !File::H.bitboard()) << 9),
        Color::Black => ((pawns & !File::A.bitboard()) >> 9) | ((pawns & !File::H.bitboard()) >> 7),
    }
}
