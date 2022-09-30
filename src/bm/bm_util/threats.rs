use cozy_chess::{BitBoard, Board, Color, File, Piece, Square};

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

fn into_pawn_threat(board: &Board, to: Square) -> bool {
    let stm = board.side_to_move();
    let potential_attackers = cozy_chess::get_pawn_attacks(to, stm);
    let opponent_pawns = board.pieces(Piece::Pawn) & board.colors(stm);
    !(potential_attackers & opponent_pawns).is_empty()
}

fn into_minor_threat(board: &Board, to: Square) -> bool {
    let stm = board.side_to_move();
    let potential_knight_attackers = cozy_chess::get_knight_moves(to);
    let opponent_knights = board.pieces(Piece::Knight);
    if !(potential_knight_attackers & opponent_knights).is_empty() {
        return true;
    }
    let potential_bishop_attackers = cozy_chess::get_bishop_moves(to, board.occupied());
    let opponent_bishops = board.pieces(Piece::Bishop) & board.colors(stm);
    !(potential_bishop_attackers & opponent_bishops).is_empty()
}

fn into_rook_threat(board: &Board, to: Square) -> bool {
    let stm = board.side_to_move();
    let potential_rook_attackers = cozy_chess::get_rook_moves(to, board.occupied());
    let opponent_rooks = board.pieces(Piece::Rook) & board.colors(stm);
    !(potential_rook_attackers & opponent_rooks).is_empty()
}

pub fn into_threat(board: &Board, piece: Piece, to: Square) -> bool {
    match piece {
        Piece::Pawn | Piece::King => false,
        Piece::Knight | Piece::Bishop => into_pawn_threat(board, to),
        Piece::Rook => into_pawn_threat(board, to) | into_minor_threat(board, to),
        Piece::Queen => {
            into_pawn_threat(board, to) | into_minor_threat(board, to) | into_rook_threat(board, to)
        }
    }
}
