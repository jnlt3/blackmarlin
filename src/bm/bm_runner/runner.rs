use std::sync::Arc;

use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_runner::config::{GuiInfo, SearchMode};
use crate::bm::bm_util::evaluator::Evaluator;
use chess::{Board, ChessMove};

use super::time::TimeManager;

pub trait Runner<Eval: 'static + Evaluator + Clone + Send> {
    fn new(board: Board, time_manager: Arc<dyn TimeManager>) -> Self;

    fn search<SM: 'static + SearchMode + Send, Info: 'static + GuiInfo + Send>(
        &mut self,
        threads: u8,
        verbose: bool,
    ) -> (ChessMove, Evaluation, u32, u32);

    fn raw_eval(&mut self) -> Evaluation;

    fn set_board(&mut self, board: Board);

    fn get_board(&self) -> &Board;

    fn make_move(&mut self, make_move: ChessMove);

    fn pv(&mut self, pv_len: usize) -> Vec<ChessMove>;
}
