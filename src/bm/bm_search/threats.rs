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

pub struct LazyThreatPos {
    threat_pos: [Option<BitBoard>; 6],
    threat_of: Color,
}

impl LazyThreatPos {
    pub fn new(threat_of: Color) -> Self {
        Self {
            threat_pos: [None; 6],
            threat_of,
        }
    }

    pub fn get(&mut self, board: &Board, piece: Piece) -> BitBoard {
        if self.threat_pos[piece as usize].is_none() {
            self.threat_pos[piece as usize] = Some(self.get_threat_pos(board, piece));
        }
        self.threat_pos[piece as usize].unwrap()
    }

    fn get_threat_pos(&self, board: &Board, piece: Piece) -> BitBoard {
        match piece {
            Piece::Pawn => pawn_threat_pos(board, self.threat_of),
            Piece::Knight => knight_threat_pos(board, self.threat_of),
            Piece::Bishop => bishop_threat_pos(board, self.threat_of),
            Piece::Rook => rook_threat_pos(board, self.threat_of),
            Piece::Queen | Piece::King => BitBoard::EMPTY,
        }
    }
}

fn pawn_threat_pos(board: &Board, threat_of: Color) -> BitBoard {
    let nthreat_of = board.colors(!threat_of);

    let pieces = board.pieces(Piece::Knight)
        | board.pieces(Piece::Bishop)
        | board.pieces(Piece::Rook)
        | board.pieces(Piece::Queen)
        | board.pieces(Piece::King);

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    for piece in pieces & nthreat_of {
        let moves = cozy_chess::get_pawn_attacks(piece, !threat_of);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}
pub fn knight_threat_pos(board: &Board, threat_of: Color) -> BitBoard {
    let nthreat_of = board.colors(!threat_of);

    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let king = board.pieces(Piece::King);

    let majors = rooks | queens | king;

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    for major in majors & nthreat_of {
        let moves = cozy_chess::get_knight_moves(major);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}

fn bishop_threat_pos(board: &Board, threat_of: Color) -> BitBoard {
    let nthreat_of = board.colors(!threat_of);

    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);
    let king = board.pieces(Piece::King);

    let majors = rooks | queens | king;

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    let bishop_blockers = board.colors(threat_of)
        | board.pieces(Piece::Pawn)
        | board.pieces(Piece::Knight)
        | board.pieces(Piece::Bishop);

    for major in majors & nthreat_of {
        let moves = cozy_chess::get_bishop_moves(major, bishop_blockers);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}

fn rook_threat_pos(board: &Board, threat_of: Color) -> BitBoard {
    let nthreat_of = board.colors(!threat_of);

    let queens = board.pieces(Piece::Queen);
    let king = board.pieces(Piece::King);

    let targets = queens | king;

    let mut level_0 = BitBoard::EMPTY;
    let mut level_1 = BitBoard::EMPTY;

    let rook_blockers = board.colors(threat_of)
        | board.pieces(Piece::Pawn)
        | board.pieces(Piece::Knight)
        | board.pieces(Piece::Bishop)
        | board.pieces(Piece::Rook);

    for major in targets & nthreat_of {
        let moves = cozy_chess::get_rook_moves(major, rook_blockers);
        level_1 |= moves & level_0;
        level_0 |= moves;
    }
    level_1
}
