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

    let w_pawn_attacks = pawn_threats(pawns & white, Color::White);
    let b_pawn_attacks = pawn_threats(pawns & black, Color::Black);

    let mut w_minor_attacks = BitBoard::EMPTY;
    let mut b_minor_attacks = BitBoard::EMPTY;
    for knight in knights & white {
        w_minor_attacks |= cozy_chess::get_knight_moves(knight);
    }
    for bishop in bishops & white {
        w_minor_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
    }
    for knight in knights & black {
        b_minor_attacks |= cozy_chess::get_knight_moves(knight);
    }
    for bishop in bishops & black {
        b_minor_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
    }

    let mut w_rook_attacks = BitBoard::EMPTY;
    let mut b_rook_attacks = BitBoard::EMPTY;
    for rook in rooks & white {
        w_rook_attacks |= cozy_chess::get_rook_moves(rook, occupied);
    }
    for rook in rooks & black {
        b_rook_attacks |= cozy_chess::get_rook_moves(rook, occupied);
    }

    (
        ((w_pawn_attacks & pieces) | (w_minor_attacks & majors) | (w_rook_attacks & queens))
            & black,
        ((b_pawn_attacks & pieces) | (b_minor_attacks & majors) | (b_rook_attacks & queens))
            & white,
    )
}

fn pawn_threats(pawns: BitBoard, color: Color) -> BitBoard {
    match color {
        Color::White => ((pawns & !File::A.bitboard()) << 7) | ((pawns & !File::H.bitboard()) << 9),
        Color::Black => ((pawns & !File::A.bitboard()) >> 9) | ((pawns & !File::H.bitboard()) >> 7),
    }
}
