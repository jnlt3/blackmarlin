use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};

use crate::bm::nnue::Nnue;

use super::{eval::Evaluation, frc};

#[derive(Debug, Clone)]
pub struct Position {
    current: Board,
    boards: Vec<Board>,
    evaluator: Nnue,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let mut evaluator = Nnue::new();
        evaluator.full_reset(&board);
        Self {
            current: board,
            boards: vec![],
            evaluator,
        }
    }

    pub fn reset(&mut self) {
        self.evaluator.full_reset(&self.current);
    }

    #[inline]
    pub fn forced_draw(&self, ply: u32) -> bool {
        if self.insufficient_material()
            || (self.half_ply() >= 100
                && (self.current.checkers().is_empty() || self.current.status() != GameStatus::Won))
        {
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
            self.evaluator.null_move();
            self.boards.push(self.current.clone());
            self.current = new_board;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn make_move(&mut self, make_move: Move) {
        self.evaluator.make_move(&self.current, make_move);
        self.boards.push(self.current.clone());
        self.current.play_unchecked(make_move);
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.evaluator.unmake_move();
        let current = self.boards.pop().unwrap();
        self.current = current;
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.board().hash()
    }

    pub fn get_eval(&mut self, stm: Color, root_eval: Evaluation) -> Evaluation {
        let piece_cnt = self.board().occupied().popcnt() as i16;

        let clamped_eval = root_eval.raw().clamp(-100, 100);
        let eval_bonus = if self.board().side_to_move() == stm {
            piece_cnt * clamped_eval / 50
        } else {
            -piece_cnt * clamped_eval / 50
        };

        let frc_score = frc::frc_corner_bishop(self.board());

        let mut score =
            self.evaluator.feed_forward(self.board().side_to_move()) + frc_score + eval_bonus;
        if is_ocb(self.board()) {
            score /= 4;
        }
        Evaluation::new(score)
    }

    pub fn insufficient_material(&self) -> bool {
        let rooks = self.current.pieces(Piece::Rook);
        let queens = self.current.pieces(Piece::Queen);
        let pawns = self.current.pieces(Piece::Pawn);
        match self.current.occupied().popcnt() {
            2 => true,
            3 => (rooks | queens | pawns).is_empty(),
            _ => false,
        }
    }
}

fn is_ocb(board: &Board) -> bool {
    const WHITE: BitBoard = BitBoard(0xAA55AA55AA55AA55);
    let kings = board.pieces(Piece::King);
    let pawns = board.pieces(Piece::Pawn);
    let bishops = board.pieces(Piece::Bishop);
    if board.occupied() == (kings | pawns | bishops) {
        let w_bishops = board.colors(Color::White) & bishops;
        let b_bishops = board.colors(Color::Black) & bishops;
        if !(w_bishops.is_empty() || b_bishops.is_empty()) {
            return ((w_bishops & WHITE).is_empty() && (b_bishops & !WHITE).is_empty())
                || ((w_bishops & !WHITE).is_empty() && (b_bishops & WHITE).is_empty());
        }
    }
    false
}
