use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};

use crate::bm::nnue::Nnue;

use super::{
    eval::Evaluation,
    frc,
    threats::{threats, ThreatOffense},
};

#[derive(Debug, Clone)]
pub struct Position {
    current: Board,
    threats: ThreatOffense,
    boards: Vec<Board>,
    threats_stack: Vec<ThreatOffense>,
    evaluator: Nnue,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let mut evaluator = Nnue::new();
        let threats = threats(&board);
        evaluator.full_reset(&board, threats.w_threats, threats.b_threats);
        Self {
            current: board,
            threats: threats,
            boards: vec![],
            threats_stack: vec![],
            evaluator,
        }
    }

    pub fn set_board(&mut self, board: Board) {
        let threats = threats(&board);
        self.evaluator
            .full_reset(&board, threats.w_threats, threats.b_threats);
        self.threats = threats;
        self.current = board;
        self.boards.clear();
    }

    pub fn reset(&mut self) {
        self.evaluator.full_reset(
            &self.current,
            self.threats.w_threats,
            self.threats.b_threats,
        );
    }

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

    pub fn half_ply(&self) -> u8 {
        self.current.halfmove_clock()
    }

    pub fn null_move(&mut self) -> bool {
        if let Some(new_board) = self.board().null_move() {
            self.evaluator.null_move();
            self.boards.push(self.current.clone());
            self.threats_stack.push(self.threats);
            self.current = new_board;
            true
        } else {
            false
        }
    }

    pub fn make_move(&mut self, make_move: Move) {
        let old_board = self.current.clone();
        let old_threats = self.threats;

        self.current.play_unchecked(make_move);
        self.threats = threats(&self.current);

        self.evaluator.make_move(
            &old_board,
            make_move,
            self.threats.w_threats,
            self.threats.b_threats,
            old_threats.w_threats,
            old_threats.b_threats,
        );

        self.boards.push(old_board);
        self.threats_stack.push(old_threats);
    }

    pub fn unmake_move(&mut self) {
        self.evaluator.unmake_move();
        let current = self.boards.pop().unwrap();
        self.threats = self.threats_stack.pop().unwrap();
        self.current = current;
    }

    pub fn hash(&self) -> u64 {
        self.board().hash()
    }

    pub fn threats(&self) -> (BitBoard, BitBoard) {
        (self.threats.w_threats, self.threats.b_threats)
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
