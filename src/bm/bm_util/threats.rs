use cozy_chess::{BitBoard, Board, Color, File, Piece};

#[derive(Debug, Copy, Clone)]
pub struct Threats {
    w_threats: BitBoard,
    b_threats: BitBoard,
    w_slider_threats: BitBoard,
    b_slider_threats: BitBoard,
}

impl Threats {
    pub fn all(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.w_threats | self.w_slider_threats,
            Color::Black => self.b_threats | self.b_slider_threats,
        }
    }

    pub fn slider_threats(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.w_slider_threats,
            Color::Black => self.b_slider_threats,
        }
    }
}

pub fn threats(board: &Board) -> Threats {
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

    let mut w_knight_attacks = BitBoard::EMPTY;
    let mut b_knight_attacks = BitBoard::EMPTY;

    let mut w_bishop_attacks = BitBoard::EMPTY;
    let mut b_bishop_attacks = BitBoard::EMPTY;

    if !(majors & black).is_empty() {
        for knight in knights & white {
            w_knight_attacks |= cozy_chess::get_knight_moves(knight);
        }
        for bishop in bishops & white {
            w_bishop_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
        }
    }
    if !(majors & white).is_empty() {
        for knight in knights & black {
            b_knight_attacks |= cozy_chess::get_knight_moves(knight);
        }
        for bishop in bishops & black {
            b_bishop_attacks |= cozy_chess::get_bishop_moves(bishop, occupied);
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

    let w_threats = w_pawn_attacks & pieces & black;
    let b_threats = b_pawn_attacks & pieces & white;
    let w_slider_threats = ((w_bishop_attacks & majors) | (w_rook_attacks & queens)) & black;
    let b_slider_threats = ((b_bishop_attacks & majors) | (b_rook_attacks & queens)) & white;
    Threats {
        w_threats,
        b_threats,
        w_slider_threats,
        b_slider_threats,
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
