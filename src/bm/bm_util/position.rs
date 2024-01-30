use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};

use crate::bm::nnue::Nnue;

use super::{eval::Evaluation, frc, threats::threats};

#[derive(Debug, Clone)]
pub struct Position {
    current: Board,
    w_threats: BitBoard,
    b_threats: BitBoard,
    boards: Vec<Board>,
    threats: Vec<(BitBoard, BitBoard)>,
    evaluator: Nnue,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let mut evaluator = Nnue::new();
        let (w_threats, b_threats) = threats(&board);
        evaluator.full_reset(&board, w_threats, b_threats);
        Self {
            current: board,
            w_threats,
            b_threats,
            threats: vec![],
            boards: vec![],
            evaluator,
        }
    }

    pub fn set_board(&mut self, board: Board) {
        let (w_threats, b_threats) = threats(&board);
        self.evaluator.full_reset(&board, w_threats, b_threats);
        self.w_threats = w_threats;
        self.b_threats = b_threats;
        self.current = board;
        self.boards.clear();
    }

    pub fn reset(&mut self) {
        self.evaluator
            .full_reset(&self.current, self.w_threats, self.b_threats);
    }

    pub fn forced_draw(&self, ply: u32) -> bool {
        if self.insufficient_material()
            || (self.fmr_plies() >= 100
                && (self.current.checkers().is_empty() || self.current.status() != GameStatus::Won))
        {
            return true;
        }
        let hash = self.hash();
        self.boards
            .iter()
            .rev()
            .take(ply as usize - 1)
            .any(|board| board.hash() == hash)
            || self
                .boards
                .iter()
                .rev()
                .skip(ply as usize - 1)
                .filter(|board| board.hash() == hash)
                .count()
                >= 2
    }

    pub fn board(&self) -> &Board {
        &self.current
    }

    pub fn fmr_plies(&self) -> u8 {
        self.current.halfmove_clock()
    }

    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board().null_move() {
            self.evaluator.null_move();
            self.boards.push(self.current.clone());
            self.threats.push((self.w_threats, self.b_threats));
            self.current = new_board;
            true
        } else {
            false
        }
    }

    pub fn make_move(&mut self, make_move: Move) {
        let old_board = self.current.clone();
        let old_w_threats = self.w_threats;
        let old_b_threats = self.b_threats;

        self.current.play_unchecked(make_move);
        (self.w_threats, self.b_threats) = threats(&self.current);

        self.evaluator.make_move(
            &old_board,
            &self.current,
            make_move,
            self.w_threats,
            self.b_threats,
            old_w_threats,
            old_b_threats,
        );

        self.boards.push(old_board);
        self.threats.push((old_w_threats, old_b_threats));
    }

    pub fn unmake_move(&mut self) {
        self.evaluator.unmake_move();
        let current = self.boards.pop().unwrap();
        (self.w_threats, self.b_threats) = self.threats.pop().unwrap();
        self.current = current;
    }

    pub fn hash(&self) -> u64 {
        self.board().hash()
    }

    pub fn threats(&self) -> (BitBoard, BitBoard) {
        (self.w_threats, self.b_threats)
    }

    pub fn get_eval(&mut self, stm: Color, root_eval: Evaluation) -> Evaluation {
        let piece_cnt = self.board().occupied().len() as i16;

        let clamped_eval = root_eval.raw().clamp(-100, 100);
        let eval_bonus = if self.board().side_to_move() == stm {
            piece_cnt * clamped_eval / 50
        } else {
            -piece_cnt * clamped_eval / 50
        };

        let frc_score = frc::frc_corner_bishop(self.board());

        Evaluation::new(
            self.evaluator
                .feed_forward(self.board().side_to_move(), piece_cnt as usize)
                + frc_score
                + eval_bonus,
        )
    }

    pub fn insufficient_material(&self) -> bool {
        let rooks = self.current.pieces(Piece::Rook);
        let queens = self.current.pieces(Piece::Queen);
        let pawns = self.current.pieces(Piece::Pawn);
        match self.current.occupied().len() {
            2 => true,
            3 => (rooks | queens | pawns).is_empty(),
            _ => false,
        }
    }
}
