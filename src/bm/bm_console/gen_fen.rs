use crate::bm::bm_eval::{eval::Depth::Next, evaluator::StdEvaluator};
use chess::{Board, ChessMove, Color, MoveGen, EMPTY};
use regex::Regex;

use crate::bm::{
    bm_eval::{eval::Evaluation, evaluator::EvalTrace},
    bm_util::position::Position,
};

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub trace: EvalTrace,
    pub result: f64,
    pub weight: f64,
}

pub fn fens(pgns: &str, weight: f64) -> String {
    let mut eval = StdEvaluator::new();
    let mut data_points = vec![];
    let pgns = pgns.split("]\n\n").collect::<Vec<_>>();
    for pgn in pgns {
        let pgn_eval = evaluate_pgn(pgn);
        data_points.extend(pgn_eval);
    }
    let mut output = "".to_string();
    for (index, (board, result)) in data_points.iter().enumerate() {
        println!("{}", index);
        let out = match result {
            Some(Color::White) => 1.0,
            Some(Color::Black) => 0.0,
            None => 0.5,
        };
        let (_, q_board) =
            small_q_search(*board, 30, Evaluation::min(), Evaluation::max(), &mut eval);
        output += &format!("{}, {}, {}\n", q_board.to_string(), out, weight);
    }
    output
}

fn small_q_search(
    board: Board,
    depth: u8,
    mut alpha: Evaluation,
    beta: Evaluation,
    eval: &mut StdEvaluator,
) -> (Evaluation, Board) {
    if depth == 0 {
        return (eval.evaluate(&board), board);
    }

    let mut best_board = board;

    if *board.checkers() == EMPTY {
        let stand_pat = eval.evaluate(&board);
        if stand_pat > alpha {
            alpha = stand_pat;
            if stand_pat >= beta {
                return (stand_pat, best_board);
            }
        }
    }

    let mut best = None;
    let mut move_gen = MoveGen::new_legal(&board);
    move_gen.set_iterator_mask(*board.combined());

    for make_move in move_gen {
        let new_board = board.make_move_new(make_move);

        let (eval, next_best) =
            small_q_search(new_board, depth - 1, beta >> Next, alpha >> Next, eval);
        let eval = eval << Next;
        if best.is_none() || eval > best.unwrap() {
            best = Some(eval);
        }
        if eval > alpha {
            alpha = eval;
            best_board = next_best;
            if eval >= beta {
                return (eval, best_board);
            }
        }
    }

    (best.unwrap_or(alpha), best_board)
}

fn evaluate_pgn(pgn: &str) -> Vec<(Board, Option<Color>)> {
    let regex = Regex::new("(O[-O]+)|([a-zA-Z]+[1-9])").unwrap();
    let new_line_split = pgn.split("\n").collect::<Vec<_>>();

    let mut game = "".to_string();
    let mut result = None;
    for line in new_line_split {
        let line = line.trim();
        if line.trim().starts_with("[") {
            continue;
        }
        game += line;
    }
    if game.contains("1-0") {
        result = Some(Color::White);
    } else if game.contains("0-1") {
        result = Some(Color::Black);
    }

    let mut boards = vec![];

    let mut position = Position::new(Board::default());
    let captures = regex.find_iter(&game);

    for c in captures {
        let make_move = ChessMove::from_san(position.board(), c.as_str());
        if make_move.is_err() {
            return vec![];
        }
        let make_move = make_move.unwrap();
        position.make_move(make_move);

        let board = *position.board();
        boards.push((board, result));
    }
    boards
}
