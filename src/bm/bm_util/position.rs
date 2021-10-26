use chess::{Board, ChessMove};

use crate::bm::bm_eval::{eval::Evaluation, evaluator::StdEvaluator};

#[derive(Debug, Clone)]
pub struct Position {
    current: Board,
    boards: Vec<Board>,
    evaluator: StdEvaluator,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let evaluator = StdEvaluator::new();
        Self {
            current: board,
            boards: vec![],
            evaluator,
        }
    }

    #[inline]
    pub fn forced_draw(&self) -> bool {
        if self.evaluator.insufficient_material(self.board()) {
            return true;
        }
        let hash = self.hash();
        self.boards
            .iter()
            .rev()
            .skip(1)
            .filter(|board| board.get_hash() == hash)
            .count()
            >= 2
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board().null_move() {
            self.boards.push(self.current);
            self.current = new_board;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn make_move(&mut self, make_move: ChessMove) {
        let old_board = *self.board();
        self.boards.push(old_board);
        old_board.make_move(make_move, &mut self.current);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.boards.pop().unwrap();
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.board().get_hash()
    }

    pub fn get_eval(&mut self) -> Evaluation {
        let board = *self.board();
        self.evaluator.evaluate(&board)
    }
}
