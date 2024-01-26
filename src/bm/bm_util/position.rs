use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};

use crate::bm::nnue::Nnue;

use super::{eval::Evaluation, frc, threats::Threats};

#[derive(Debug, Clone)]
pub struct Position {
    current: Board,
    threat: Threats,
    boards: Vec<Board>,
    threats: Vec<Threats>,
    evaluator: Nnue,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let mut evaluator = Nnue::new();
        let threats = Threats::new(&board);

        evaluator.full_reset(&board, threats);
        Self {
            current: board,
            threat: threats,
            threats: vec![],
            boards: vec![],
            evaluator,
        }
    }

    pub fn set_board(&mut self, board: Board) {
        let threats = Threats::new(&board);
        self.evaluator.full_reset(&board, threats);
        self.threat = threats;
        self.current = board;
        self.boards.clear();
    }

    pub fn reset(&mut self) {
        self.evaluator.full_reset(&self.current, self.threat);
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
            self.threats.push(self.threat);
            self.current = new_board;
            true
        } else {
            false
        }
    }

    pub fn make_move(&mut self, make_move: Move) {
        let old_board = self.current.clone();
        let old_threats = self.threat;

        self.current.play_unchecked(make_move);
        self.threat = Threats::new(&self.current);

        self.evaluator
            .make_move(&old_board, make_move, self.threat, old_threats);

        self.boards.push(old_board);
        self.threats.push(old_threats);
    }

    pub fn unmake_move(&mut self) {
        self.evaluator.unmake_move();
        let current = self.boards.pop().unwrap();
        self.threat = self.threats.pop().unwrap();
        self.current = current;
    }

    pub fn hash(&self) -> u64 {
        self.board().hash()
    }

    pub fn threats(&self) -> (BitBoard, BitBoard) {
        (
            self.threat.from_color(Color::White),
            self.threat.from_color(Color::Black),
        )
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
