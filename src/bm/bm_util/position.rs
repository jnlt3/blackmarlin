use cozy_chess::{Board, Move, Piece};

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
    pub fn forced_draw(&self, ply: u32) -> bool {
        if self.evaluator.insufficient_material(self.board()) || self.half_ply() >= 100 {
            return true;
        }
        let hash = self.hash();
        self.boards
            .iter()
            .rev()
            .skip(1)
            .take(ply as usize)
            .any(|board| board.hash() == hash)
            || self
                .boards
                .iter()
                .rev()
                .skip(ply as usize + 1)
                .filter(|board| board.hash() == hash)
                .count()
                >= 2
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn half_ply(&self) -> u8 {
        self.current.halfmove_clock()
    }

    #[inline]
    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board().null_move() {
            self.boards.push(self.current.clone());
            self.current = new_board;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn make_move(&mut self, make_move: Move) {
        self.boards.push(self.current.clone());
        self.current.play_unchecked(make_move);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        let current = self.boards.pop().unwrap();
        self.current = current;
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.board().hash()
    }

    pub fn get_eval(&mut self) -> Evaluation {
        let board = self.board().clone();
        self.evaluator.evaluate(&board)
    }
}
