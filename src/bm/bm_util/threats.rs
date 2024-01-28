use cozy_chess::{BitBoard, Board, Color, File, Piece};

#[derive(Debug, Clone, Copy)]
pub struct ThreatOffense {
    pub w_threats: BitBoard,
    pub b_threats: BitBoard,
    pub w_offense: BitBoard,
    pub b_offense: BitBoard,
}

pub fn threats(board: &Board) -> ThreatOffense {
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

    let w_attacks =
        (w_pawn_attacks & pieces) | (w_minor_attacks & majors) | (w_rook_attacks & queens);
    let b_attacks =
        (b_pawn_attacks & pieces) | (b_minor_attacks & majors) | (b_rook_attacks & queens);

    let w_king = cozy_chess::get_king_moves(board.king(Color::White));
    let b_king = cozy_chess::get_king_moves(board.king(Color::Black));

    ThreatOffense {
        w_threats: w_attacks & black,
        b_threats: b_attacks & white,
        w_offense: (w_pawn_attacks | w_minor_attacks | w_rook_attacks) & b_king,
        b_offense: (b_pawn_attacks | b_minor_attacks | b_rook_attacks) & w_king,
    }
}

fn pawn_threats(pawns: BitBoard, color: Color) -> BitBoard {
    let threats = match color {
        Color::White => {
            ((pawns & !File::A.bitboard()).0 << 7) | ((pawns & !File::H.bitboard()).0 << 9)
        }
        Color::Black => {
            ((pawns & !File::A.bitboard()).0 >> 9) | ((pawns & !File::H.bitboard()).0 >> 7)
        }
    };
    BitBoard(threats)
}
