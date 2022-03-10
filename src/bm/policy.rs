use cozy_chess::{Board, Color, Move};

include!(concat!(env!("OUT_DIR"), "/policy_weights.rs"));

pub fn move_eval(board: &Board, make_move: Move) -> i16 {
    let move_piece = board.piece_on(make_move.from).unwrap() as usize;
    let move_sq = match board.side_to_move() {
        Color::White => make_move.to as usize,
        Color::Black => make_move.to as usize ^ 56,
    };
    let move_index = move_piece * 64 + move_sq;

    let mut score_0 = BIAS_0[move_index] as i16;
    for sq in board.occupied() {
        let piece = board.piece_on(sq).unwrap() as usize;
        let color = board.color_on(sq).unwrap();
        let color = match board.side_to_move() {
            Color::White => color as usize,
            Color::Black => (!color) as usize,
        };
        let sq = match board.side_to_move() {
            Color::White => sq as usize,
            Color::Black => sq as usize ^ 56,
        };

        let piece_index = sq + (piece + color * 6) * 64;
        score_0 += WEIGHTS_0[piece_index * 384 + move_index] as i16;
    }
    -((score_0 as i64) * 170 / 64) as i16
}
