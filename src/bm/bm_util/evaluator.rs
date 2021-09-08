use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_util::position::Position;
use chess::{Board, ChessMove};

pub trait Evaluator: 'static + Clone + Send {
    fn new() -> Self;

    fn see(board: Board, make_move: ChessMove) -> i32;

    fn evaluate(&mut self, position: &Position) -> Evaluation;

    fn clear_cache(&mut self);
}
