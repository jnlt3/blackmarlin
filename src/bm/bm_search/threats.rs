use cozy_chess::{BitBoard, Board, Color, Piece};

pub fn threats(board: &Board, threats_of: Color) -> BitBoard {
    let occupied = board.occupied();
    let color = board.colors(threats_of);
    let n_color = board.colors(!threats_of);

    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);

    let minors = knights | bishops;
    let majors = rooks | queens;
    let pieces = minors | majors;

    let mut pawn_attacks = BitBoard::EMPTY;
    for pawn in pawns & color {
        pawn_attacks |= cozy_chess::get_pawn_attacks(pawn, threats_of);
    }

    let mut minor_attacks = BitBoard::EMPTY;
    for knight in knights & color {
        minor_attacks |= cozy_chess::get_knight_moves(knight);
    }

    for bishop in bishops & color {
        minor_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
    }

    let mut rook_attacks = BitBoard::EMPTY;
    for rook in rooks & color {
        rook_attacks |= cozy_chess::get_rook_moves(rook, occupied);
    }

    ((pawn_attacks & pieces) | (minor_attacks & majors) | (rook_attacks & queens)) & n_color
}

pub fn promo_threats(board: &Board, threat_of: Color) -> BitBoard {
    let occupied = board.occupied();
    let opponent = board.colors(!threat_of);

    let pawns = board.pieces(Piece::Pawn);
    let stm_pawns = pawns & board.colors(threat_of);

    let mut threats = BitBoard::EMPTY;
    for pawn in stm_pawns {
        threats |= cozy_chess::get_pawn_quiets(pawn, threat_of, occupied)
            | (cozy_chess::get_pawn_attacks(pawn, threat_of) & opponent);
    }
    threats
}
