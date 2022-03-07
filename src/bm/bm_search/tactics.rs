use cozy_chess::{Board, Color, Piece};

const P2N_T: i16 = 300;
const P2B_T: i16 = 300;
const P2R_T: i16 = 500;
const P2Q_T: i16 = 900;

pub fn get_pawn_threats(board: &Board) -> (i16, i16) {
    let white = board.colors(Color::White);
    let black = board.colors(Color::Black);

    let pawns = board.pieces(Piece::Pawn);
    let knights = board.pieces(Piece::Knight);
    let bishops = board.pieces(Piece::Bishop);
    let rooks = board.pieces(Piece::Rook);
    let queens = board.pieces(Piece::Rook);

    let mut w_threats = 0;
    let mut b_threats = 0;

    for pawn in white & pawns {
        let attacks = cozy_chess::get_pawn_attacks(pawn, Color::White) & black;
        w_threats += (attacks & knights).popcnt() as i16 * P2N_T;
        w_threats += (attacks & bishops).popcnt() as i16 * P2B_T;
        w_threats += (attacks & rooks).popcnt() as i16 * P2R_T;
        w_threats += (attacks & queens).popcnt() as i16 * P2Q_T;
    }

    for pawn in black & pawns {
        let attacks = cozy_chess::get_pawn_attacks(pawn, Color::Black) & white;
        b_threats += (attacks & knights).popcnt() as i16 * P2N_T;
        b_threats += (attacks & bishops).popcnt() as i16 * P2B_T;
        b_threats += (attacks & rooks).popcnt() as i16 * P2R_T;
        b_threats += (attacks & queens).popcnt() as i16 * P2Q_T;
    }
    (w_threats, b_threats)
}
