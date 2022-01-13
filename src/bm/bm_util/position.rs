use cozy_chess::{Board, Color, Move, Piece};

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

    pub fn material(&self) -> Evaluation {
        let white = self.board().colors(Color::White);
        let black = self.board().colors(Color::Black);
        let pawns = self.board().pieces(Piece::Pawn);
        let knights = self.board().pieces(Piece::Knight);
        let bishops = self.board().pieces(Piece::Bishop);
        let rooks = self.board().pieces(Piece::Rook);
        let queens = self.board().pieces(Piece::Queen);
        let pawn_diff = (white & pawns).popcnt() as i16 - (black & pawns).popcnt() as i16;
        let knight_diff = (white & knights).popcnt() as i16 - (black & knights).popcnt() as i16;
        let bishop_diff = (white & bishops).popcnt() as i16 - (black & bishops).popcnt() as i16;
        let rook_diff = (white & rooks).popcnt() as i16 - (black & rooks).popcnt() as i16;
        let queen_diff = (white & queens).popcnt() as i16 - (black & queens).popcnt() as i16;
        let mat = pawn_diff + (knight_diff + bishop_diff) * 3 + rook_diff * 5 + queen_diff * 9;
        let turn = match self.board().side_to_move() {
            Color::White => 100,
            Color::Black => -100,
        };
        Evaluation::new(turn * mat)
    }
}
