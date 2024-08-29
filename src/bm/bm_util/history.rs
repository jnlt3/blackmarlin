use cozy_chess::{Color, Move, Piece, Square};

use crate::bm::bm_runner::ab_runner::MoveData;

use super::position::Position;
use super::table_types::{new_butterfly_table, new_piece_to_table, Butterfly, PieceTo};

pub const MAX_HIST: i16 = 512;

pub const CORR_HIST_GRAIN: i32 = 256;
pub const MAX_CORRECT: i32 = 32;
pub const MAX_DIFF: i32 = 512;

fn hist_stat(amt: i16) -> i16 {
    (amt * 13).min(MAX_HIST)
}

fn bonus(hist: &mut i16, amt: i16) {
    let change = hist_stat(amt);
    let decay = (change as i32 * (*hist) as i32 / MAX_HIST as i32) as i16;
    let increment = change - decay;
    *hist += increment;
}

fn malus(hist: &mut i16, amt: i16) {
    let change = hist_stat(amt);
    let decay = (change as i32 * (*hist) as i32 / MAX_HIST as i32) as i16;
    let decrement = change + decay;
    *hist -= decrement;
}

/// Contains information calculated to index the history tables
#[derive(Copy, Clone)]
pub struct HistoryIndices {
    cont_mv_1: Option<(Piece, Square)>,
    cont_mv_2: Option<(Piece, Square)>,
    cont_mv_4: Option<(Piece, Square)>,
}

impl HistoryIndices {
    pub fn new(
        cont_mv_1: Option<MoveData>,
        cont_mv_2: Option<MoveData>,
        cont_mv_4: Option<MoveData>,
    ) -> Self {
        let convert = |mv: MoveData| (mv.piece, mv.to);
        let cont_mv_1 = cont_mv_1.map(convert);
        let cont_mv_2 = cont_mv_2.map(convert);
        let cont_mv_4 = cont_mv_4.map(convert);
        Self {
            cont_mv_1,
            cont_mv_2,
            cont_mv_4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct History {
    quiet: Box<[[Butterfly<i16>; 2]; Color::NUM]>,
    capture: Box<[[Butterfly<i16>; 2]; Color::NUM]>,
    counter_move: Box<[PieceTo<PieceTo<i16>>; Color::NUM]>,
    followup_move: Box<[PieceTo<PieceTo<i16>>; Color::NUM]>,

    pawn_corr: Box<[[i32; u16::MAX as usize + 1]; Color::NUM]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            quiet: Box::new([[new_butterfly_table(0); Color::NUM]; 2]),
            capture: Box::new([[new_butterfly_table(0); Color::NUM]; 2]),
            counter_move: Box::new([new_piece_to_table(new_piece_to_table(0)); Color::NUM]),
            followup_move: Box::new([new_piece_to_table(new_piece_to_table(0)); Color::NUM]),
            pawn_corr: Box::new([[0; u16::MAX as usize + 1]; Color::NUM]),
        }
    }

    /// Returns quiet history value for the given move
    pub fn get_quiet(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        let (_, nstm_threats) = pos.threats();
        self.quiet[stm as usize][nstm_threats.has(make_move.from) as usize][make_move.from as usize]
            [make_move.to as usize]
    }

    fn get_quiet_mut(&mut self, pos: &Position, make_move: Move) -> &mut i16 {
        let stm = pos.board().side_to_move();
        let (_, nstm_threats) = pos.threats();
        &mut self.quiet[stm as usize][nstm_threats.has(make_move.from) as usize]
            [make_move.from as usize][make_move.to as usize]
    }

    /// Returns capture history value for the given move
    pub fn get_capture(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        let (_, nstm_threats) = pos.threats();
        self.capture[stm as usize][nstm_threats.has(make_move.from) as usize]
            [make_move.from as usize][make_move.to as usize]
    }

    fn get_capture_mut(&mut self, pos: &Position, make_move: Move) -> &mut i16 {
        let stm = pos.board().side_to_move();
        let (_, nstm_threats) = pos.threats();
        &mut self.capture[stm as usize][nstm_threats.has(make_move.from) as usize]
            [make_move.from as usize][make_move.to as usize]
    }

    /// Returns None if a previous move isn't available
    ///
    /// Recommended to .unwrap_or_default()
    ///
    /// Do not use for captures
    pub fn get_counter_move(
        &self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<i16> {
        let (prev_piece, prev_to) = indices.cont_mv_1?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            self.counter_move[stm as usize][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    fn get_counter_move_mut(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<&mut i16> {
        let (prev_piece, prev_to) = indices.cont_mv_1?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            &mut self.counter_move[stm as usize][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    /// Returns None if a previous move isn't available
    ///
    /// Recommended to .unwrap_or_default()
    ///
    /// Do not use for captures
    pub fn get_followup_move(
        &self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<i16> {
        let (prev_piece, prev_to) = indices.cont_mv_2?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            self.followup_move[stm as usize][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    fn get_followup_move_mut(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<&mut i16> {
        let (prev_piece, prev_to) = indices.cont_mv_2?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            &mut self.followup_move[stm as usize][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    /// Returns None if a previous move isn't available
    ///
    /// Recommended to .unwrap_or_default()
    ///
    /// Do not use for captures
    pub fn get_followup_move_2(
        &self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<i16> {
        let (prev_piece, prev_to) = indices.cont_mv_4?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            self.followup_move[stm as usize][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    fn get_followup_move_2_mut(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<&mut i16> {
        let (prev_piece, prev_to) = indices.cont_mv_4?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            &mut self.followup_move[stm as usize][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    /// If the cut-off move is a capture, the cut-off move is given a bonus in
    /// capture history and the other captures are given maluses in capture history
    ///
    /// If the cut-off move is a quiet, the cut-off move is given a bonus in all
    /// quiet histories (main quiet history, followup move history, counter move history)
    /// captures are given maluses in capture-history
    pub fn update_history(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        cutoff_move: Move,
        quiets: &[Move],
        captures: &[Move],
        amt: i16,
    ) {
        let is_capture = pos
            .board()
            .colors(!pos.board().side_to_move())
            .has(cutoff_move.to);
        if !is_capture {
            self.update_quiet(pos, indices, cutoff_move, quiets, amt);
        } else {
            bonus(self.get_capture_mut(pos, cutoff_move), amt);
        }
        for &failed_move in captures {
            malus(self.get_capture_mut(pos, failed_move), amt);
        }
    }

    fn update_quiet(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
        fails: &[Move],
        amt: i16,
    ) {
        bonus(self.get_quiet_mut(pos, make_move), amt);
        for &failed_move in fails {
            malus(self.get_quiet_mut(pos, failed_move), amt);
        }
        if let Some(counter_move_hist) = self.get_counter_move_mut(pos, indices, make_move) {
            bonus(counter_move_hist, amt);
            for &failed_move in fails {
                let failed_hist = self
                    .get_counter_move_mut(pos, indices, failed_move)
                    .unwrap();
                malus(failed_hist, amt);
            }
        }
        if let Some(followup_move_hist) = self.get_followup_move_mut(pos, indices, make_move) {
            bonus(followup_move_hist, amt);
            for &failed_move in fails {
                let failed_hist = self
                    .get_followup_move_mut(pos, indices, failed_move)
                    .unwrap();
                malus(failed_hist, amt);
            }
        }
        if let Some(followup_move_hist_2) = self.get_followup_move_2_mut(pos, indices, make_move) {
            bonus(followup_move_hist_2, amt);
            for &failed_move in fails {
                let failed_hist = self
                    .get_followup_move_2_mut(pos, indices, failed_move)
                    .unwrap();
                malus(failed_hist, amt);
            }
        }
    }

    fn update_corr(val: &mut i32, eval_diff: i16, depth: u32) {
        let weight = (depth * 8).min(128) as i32;
        let new_value = *val + (eval_diff as i32).clamp(-MAX_DIFF, MAX_DIFF) * weight;
        *val = new_value.clamp(
            -MAX_CORRECT * CORR_HIST_GRAIN,
            MAX_CORRECT * CORR_HIST_GRAIN,
        );
    }

    pub fn update_corr_hist(&mut self, pos: &Position, eval_diff: i16, depth: u32) {
        let stm = pos.board().side_to_move();
        let hash = pos.pawn_hash();
        Self::update_corr(
            &mut self.pawn_corr[stm as usize][hash as usize],
            eval_diff,
            depth,
        );
    }

    pub fn get_correction(&self, pos: &Position) -> i16 {
        let stm = pos.board().side_to_move();
        let hash = pos.pawn_hash();
        (self.pawn_corr[stm as usize][hash as usize] / CORR_HIST_GRAIN) as i16
    }
}
