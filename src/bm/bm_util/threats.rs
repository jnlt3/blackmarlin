use cozy_chess::{BitBoard, Board, Color, File, Move, Piece};

pub fn recalculate_threats(
    board: &Board,
    make_move: Move,
    w_threats: BitBoard,
    b_threats: BitBoard,
) -> bool {
    //Don't allow any promotions, captures or castles through as they are too complicated to handle
    if make_move.promotion.is_some() || board.occupied().has(make_move.to) {
        return true;
    }
    let stm = board.side_to_move();

    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Queen);

    let minors = knights & bishops;

    let orthogonals = rooks | queens;
    let diagonals = bishops | queens;

    let occupied = board.occupied();

    let nstm_pieces = board.colors(!stm);

    let (stm_threats, nstm_threats) = match stm {
        Color::White => (w_threats, b_threats),
        Color::Black => (b_threats, w_threats),
    };
    // If we are moving a threatened piece, threats will change
    if nstm_threats.has(make_move.from) {
        return true;
    }
    // If we are capturing a threatened piece, threats will change
    if stm_threats.has(make_move.to) {
        return true;
    }

    // If we are opening up space for sliding pieces, threats might change
    let discovery_diag = cozy_chess::get_bishop_moves(make_move.from, occupied) & diagonals;
    let discovery_ortho = cozy_chess::get_rook_moves(make_move.from, occupied) & orthogonals;

    // If we are blocking sliding pieces, threats might change
    let block_diag = cozy_chess::get_bishop_moves(make_move.to, occupied) & diagonals;
    let block_ortho = cozy_chess::get_rook_moves(make_move.to, occupied) & orthogonals;

    if !(discovery_diag | discovery_ortho | block_diag | block_ortho).is_empty() {
        return true;
    }

    let piece = board.piece_on(make_move.from).unwrap();
    match piece {
        Piece::Pawn => {
            if make_move.to.file() != make_move.from.file() {
                return true;
            }
            let prev_threats = cozy_chess::get_pawn_attacks(make_move.from, stm) & stm_threats;
            let new_threats =
                cozy_chess::get_pawn_attacks(make_move.to, stm) & nstm_pieces & !pawns;

            !(prev_threats.is_empty() && new_threats.is_empty())
        }
        Piece::Knight => {
            let prev_threats = cozy_chess::get_knight_moves(make_move.from) & stm_threats;
            let new_threats =
                cozy_chess::get_knight_moves(make_move.to) & nstm_pieces & !pawns & !minors;
            let into_threat = cozy_chess::get_pawn_attacks(make_move.to, stm) & nstm_pieces & pawns;
            !(prev_threats.is_empty() && new_threats.is_empty() && into_threat.is_empty())
        }
        Piece::Bishop => {
            let prev_threats = cozy_chess::get_bishop_moves(make_move.from, occupied) & stm_threats;
            let new_threats = cozy_chess::get_bishop_moves(make_move.to, occupied)
                & nstm_pieces
                & !pawns
                & !minors;
            let into_threat = cozy_chess::get_pawn_attacks(make_move.to, stm) & nstm_pieces & pawns;
            !(prev_threats.is_empty() && new_threats.is_empty() && into_threat.is_empty())
        }
        Piece::Queen | Piece::Rook => true,
        Piece::King => false,
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
