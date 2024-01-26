use cozy_chess::{BitBoard, Board, Color, File, Piece};

pub const NUM_PIECES: usize = 4;
pub const PIECES: [Piece; NUM_PIECES] = [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];

#[derive(Debug, Copy, Clone)]
pub struct Threats {
    color_threats: [BitBoard; Color::NUM],
    piece_threats: [BitBoard; NUM_PIECES],
}

impl Threats {
    pub fn new(board: &Board) -> Self {
        let (w_threats, b_threats) = threats(board);
        let all_threats = w_threats | b_threats;
        let mut piece_threats = [BitBoard::EMPTY; NUM_PIECES];
        for (piece_threat, &piece) in piece_threats.iter_mut().zip(&PIECES) {
            *piece_threat = all_threats & board.pieces(piece);
        }
        Threats {
            color_threats: [w_threats, b_threats],
            piece_threats,
        }
    }
    pub fn from_color(&self, color: Color) -> BitBoard {
        self.color_threats[color as usize]
    }

    pub fn to_piece(&self, color: Color, piece: Piece) -> BitBoard {
        (match piece {
            Piece::Pawn | Piece::King => unreachable!(),
            Piece::Knight => self.piece_threats[0],
            Piece::Bishop => self.piece_threats[1],
            Piece::Rook => self.piece_threats[2],
            Piece::Queen => self.piece_threats[3],
        }) & self.from_color(!color)
    }
}

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

    if !(majors & black).is_empty() {
        for knight in knights & white {
            w_minor_attacks |= cozy_chess::get_knight_moves(knight);
        }
        for bishop in bishops & white {
            w_minor_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
        }
    }
    if !(majors & white).is_empty() {
        for knight in knights & black {
            b_minor_attacks |= cozy_chess::get_knight_moves(knight);
        }
        for bishop in bishops & black {
            b_minor_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
        }
    }

    let mut w_rook_attacks = BitBoard::EMPTY;
    let mut b_rook_attacks = BitBoard::EMPTY;
    if !(queens & black).is_empty() {
        for rook in rooks & white {
            w_rook_attacks |= cozy_chess::get_rook_moves(rook, occupied);
        }
    }
    if !(queens & white).is_empty() {
        for rook in rooks & black {
            b_rook_attacks |= cozy_chess::get_rook_moves(rook, occupied);
        }
    }

    (
        ((w_pawn_attacks & pieces) | (w_minor_attacks & majors) | (w_rook_attacks & queens))
            & black,
        ((b_pawn_attacks & pieces) | (b_minor_attacks & majors) | (b_rook_attacks & queens))
            & white,
    )
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
