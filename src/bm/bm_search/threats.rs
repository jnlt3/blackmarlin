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

pub fn get_threat_positions(board: &Board, stm: Color, piece: Piece) -> BitBoard {
    match piece {
        Piece::Pawn => pawn_threat_pos(board, stm),
        Piece::Knight => knight_threat_pos(board, stm),
        Piece::Bishop => bishop_threat_pos(board, stm),
        Piece::Rook => rook_threat_pos(board, stm),
        Piece::Queen | Piece::King => BitBoard::EMPTY,
    }
}

fn pawn_threat_pos(board: &Board, stm: Color) -> BitBoard {
    let nstm = board.colors(!stm);

    let pieces = board.pieces(Piece::Knight)
        | board.pieces(Piece::Bishop)
        | board.pieces(Piece::Rook)
        | board.pieces(Piece::Queen)
        | board.pieces(Piece::King);

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    for piece in pieces & nstm {
        let moves = cozy_chess::get_pawn_attacks(piece, !stm);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}
pub fn knight_threat_pos(board: &Board, stm: Color) -> BitBoard {
    let nstm = board.colors(!stm);

    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let king = board.pieces(Piece::King);

    let majors = rooks | queens | king;

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    for major in majors & nstm {
        let moves = cozy_chess::get_knight_moves(major);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}

fn bishop_threat_pos(board: &Board, stm: Color) -> BitBoard {
    let nstm = board.colors(!stm);

    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let king = board.pieces(Piece::King);

    let majors = rooks | queens | king;

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    let bishop_blockers = board.colors(stm)
        | board.pieces(Piece::Pawn)
        | board.pieces(Piece::Knight)
        | board.pieces(Piece::Bishop);

    for major in majors & nstm {
        let moves = cozy_chess::get_bishop_moves(major, bishop_blockers);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}

fn rook_threat_pos(board: &Board, stm: Color) -> BitBoard {
    let nstm = board.colors(!stm);

    let queens = board.pieces(Piece::Queen);
    let king = board.pieces(Piece::King);

    let targets = queens | king;

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    let rook_blockers = board.colors(stm)
        | board.pieces(Piece::Pawn)
        | board.pieces(Piece::Knight)
        | board.pieces(Piece::Bishop)
        | board.pieces(Piece::Rook);

    for major in targets & nstm {
        let moves = cozy_chess::get_rook_moves(major, rook_blockers);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}
