use cozy_chess::{BitBoard, Board, Color, Piece};

pub struct Threats {
    pub all: BitBoard,
    pub strong: BitBoard,
}

pub fn defended(board: &Board, side: Color) -> BitBoard {
    let mut defended = BitBoard::EMPTY;

    let occupied = board.occupied();
    let color = board.colors(side);
    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);

    for pawn in pawns & color {
        defended |= cozy_chess::get_pawn_attacks(pawn, side);
    }
    for knight in knights & color {
        defended |= cozy_chess::get_knight_moves(knight);
    }
    for bishop in (bishops | queens) & color {
        defended |= cozy_chess::get_bishop_moves(bishop, occupied);
    }
    for rook in (rooks | queens) & color {
        defended |= cozy_chess::get_rook_moves(rook, occupied);
    }
    defended
}

pub fn threats(board: &Board, threats_of: Color, defended: BitBoard) -> Threats {
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

    let strong_threats =
        ((pawn_attacks & pieces) | (minor_attacks & majors) | (rook_attacks & queens)) & n_color;
    let weak_threats = (pawn_attacks | minor_attacks | rook_attacks) & n_color & !defended;
    Threats {
        all: weak_threats | strong_threats,
        strong: strong_threats,
    }
}
