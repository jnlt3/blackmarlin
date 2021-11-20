use chess::{Board, ChessMove, Piece};

use crate::bm::bm_eval::{eval::Evaluation, evaluator::StdEvaluator};

#[derive(Debug, Clone)]
pub struct Position {
    current: Board,
    half_ply: u8,
    boards: Vec<(Board, u8)>,
    evaluator: StdEvaluator,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let evaluator = StdEvaluator::new();
        Self {
            current: board,
            half_ply: 0_u8,
            boards: vec![],
            evaluator,
        }
    }

    #[inline]
    pub fn forced_draw(&self) -> bool {
        if self.evaluator.insufficient_material(self.board()) || self.half_ply() >= 100 {
            return true;
        }
        let hash = self.hash();
        self.boards
            .iter()
            .rev()
            .skip(1)
            .filter(|board| board.0.get_hash() == hash)
            .count()
            >= 2
    }

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn half_ply(&self) -> u8 {
        self.half_ply
    }

    #[inline]
    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board().null_move() {
            self.boards.push((self.current, self.half_ply + 1));
            self.current = new_board;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn make_move(&mut self, make_move: ChessMove) {
        let old_board = self.current;
        self.boards.push((self.current, self.half_ply));
        old_board.make_move(make_move, &mut self.current);
        if old_board.piece_on(make_move.get_dest()).is_some()
            || old_board.piece_on(make_move.get_source()).unwrap() == Piece::Pawn
        {
            self.half_ply = 0;
        } else {
            self.half_ply += 1;
        }
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        let (current, half_ply) = self.boards.pop().unwrap();
        self.current = current;
        self.half_ply = half_ply;
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
