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

    /// Clears position history, sets board as current root
    /// Forces recalculation of NNUE accumulators and threats
    pub fn set_board(&mut self, board: Board) {
        let (w_threats, b_threats) = threats(&board);
        self.evaluator.full_reset(&board, w_threats, b_threats);
        self.w_threats = w_threats;
        self.b_threats = b_threats;
        self.current = board;
        self.boards.clear();
    }

    /// Forces recalculation of NNUE accumulators
    pub fn reset(&mut self) {
        self.evaluator
            .full_reset(&self.current, self.w_threats, self.b_threats);
    }

    /// Returns true for 50 move rule and three fold repetitions
    ///
    /// If a two fold repetition occurs with both positions being
    /// after search root, it's considered a three fold repetition
    ///
    /// Returns true if [insufficient material](Self::insufficient_material)
    pub fn forced_draw(&self, ply: u32) -> bool {
        if self.insufficient_material()
            || (self.current.halfmove_clock() >= 100
                && (self.current.checkers().is_empty() || self.current.status() != GameStatus::Won))
        {
            return true;
        }
        let hash = self.hash();
        let two_fold = self
            .boards
            .iter()
            .rev()
            .take(ply as usize - 1)
            .any(|board| board.hash() == hash);
        let three_fold = self
            .boards
            .iter()
            .rev()
            .skip(ply as usize - 1)
            .filter(|board| board.hash() == hash)
            .count()
            >= 2;
        two_fold || three_fold
    }

    /// Current board
    pub fn board(&self) -> &Board {
        &self.current
    }

    /// Attempts to make a null move
    ///
    /// Returns false if null move can't be done
    ///
    /// Returns true if null move was played
    pub fn null_move(&mut self) -> bool {
        let Some(new_board) = self.board().null_move() else {
            return false;
        };
        self.evaluator.null_move();
        self.boards.push(self.current.clone());
        self.threats.push((self.w_threats, self.b_threats));
        self.current = new_board;
        true
    }

    /// See [Position::make_move]
    /// - Runs post_make after making the move, before updating the neural network
    pub fn make_move_fetch<F: Fn(&Board)>(&mut self, make_move: Move, post_make: F) {
        let old_board = self.current.clone();
        let old_w_threats = self.w_threats;
        let old_b_threats = self.b_threats;

        self.current.play_unchecked(make_move);
        post_make(&self.current);
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

    /// Makes move, updates accumulators and calculates threats
    /// - Expensive function, only use if the move is going to be searched
    pub fn make_move(&mut self, make_move: Move) {
        self.make_move_fetch(make_move, |_| {});
    }

    /// Takes back one (move)[Self::make_move]
    pub fn unmake_move(&mut self) {
        self.evaluator.unmake_move();
        let current = self.boards.pop().unwrap();
        (self.w_threats, self.b_threats) = self.threats.pop().unwrap();
        self.current = current;
    }

    pub fn hash(&self) -> u64 {
        self.board().hash()
    }

    /// Returns side to move relative threats
    pub fn threats(&self) -> (BitBoard, BitBoard) {
        match self.current.side_to_move() {
            Color::White => (self.w_threats, self.b_threats),
            Color::Black => (self.b_threats, self.w_threats),
        }
    }

    /// Returns aggression value
    /// - Value may vary depending on position and root evaluation
    /// - Avoid storing, instead recalculate for a given position
    pub fn aggression(&self, stm: Color, root_eval: Evaluation) -> i16 {
        let piece_cnt = self.board().occupied().len() - self.board().pieces(Piece::Pawn).len();
        let scale = 2 * piece_cnt as i16;

        let clamped_eval = root_eval.raw().clamp(-200, 200);
        (match self.board().side_to_move() == stm {
            true => scale * clamped_eval,
            false => -scale * clamped_eval,
        }) / 100
    }

    /// Calculates NN evaluation + FRC bonus
    /// - Add [aggression](Self::aggression) if using for search results & pruning
    pub fn get_eval(&mut self) -> Evaluation {
        let frc_score = frc::frc_corner_bishop(self.board());
        let piece_cnt = self.board().occupied().len() as i16;

        Evaluation::new(
            self.evaluator
                .feed_forward(self.board().side_to_move(), piece_cnt as usize)
                + frc_score,
        )
    }

    /// Handles insufficient material for the following cases:
    /// - Two kings
    /// - Two kings and one minor piece
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

    /// Returns true if a move is capture
    /// - Excludes en-passant
    pub fn is_capture(&self, mv: Move) -> bool {
        let nstm = !self.board().side_to_move();
        self.board().colors(nstm).has(mv.to)
    }

    /// Returns true if a move is not a capture or a promotion
    /// - Counts an en-passant capture as quiet
    pub fn is_quiet(&self, mv: Move) -> bool {
        mv.promotion.is_none() && !self.is_capture(mv)
    }
}
