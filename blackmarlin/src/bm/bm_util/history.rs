use std::alloc::Layout;
use std::cell::Cell;

use cozy_chess::{BitBoard, Color, Move, Piece, Square};

use crate::bm::bm_runner::ab_runner::MoveData;

use super::position::Position;
use super::table_types::{Butterfly, PieceTo, new_butterfly_table, new_piece_to_table};

pub const MAX_HIST: i16 = 512;

pub const CORR_HIST_GRAIN: i32 = 256;
pub const MAX_CORRECT: i32 = 32;

#[derive(Debug, Clone)]
struct Entry<T = i16>
where
    T: Copy,
{
    value: Cell<T>,
}

impl<T: Copy> Entry<T> {
    fn get(&self) -> T {
        self.value.get()
    }
}

impl Entry<i16> {
    fn update(&self, amt: i16) {
        // TODO: Move this into search
        let score = hist_stat(amt);
        let value = self.get();
        let decay = (score as i32 * value as i32 / MAX_HIST as i32) as i16;
        self.value.set(score * 2 - decay)
    }
}

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
struct SideHistory {
    quiet: [Butterfly<Entry>; 2],
    noisy: [Butterfly<Entry>; 2],
    counter_move: PieceTo<PieceTo<Entry>>,
    followup_move: PieceTo<PieceTo<Entry>>,
    pawn_corr: [Entry<i32>; u16::MAX as usize],
}

#[derive(Debug, Clone)]
pub struct Hist2 {
    sides: Box<[SideHistory; 2]>,
}

impl Hist2 {
    pub fn new_zeroed() -> Self {
        let sides: *mut [SideHistory; 2] =
            unsafe { std::alloc::alloc_zeroed(Layout::new::<[SideHistory; 2]>()) as *mut _ };
        assert!(!sides.is_null());
        Self {
            sides: unsafe { Box::from_raw(sides) },
        }
    }

    pub fn slice(
        &self,
        pos: &Position,
        cont_1: Option<MoveData>,
        cont_2: Option<MoveData>,
        cont_4: Option<MoveData>,
    ) -> HistSlice<'_> {
        let stm = pos.board().side_to_move();
        let side = &self.sides[stm as usize];
        let (_, nstm_threats) = pos.threats();

        let cont_1 = cont_1.map(|mv| &side.counter_move[mv.piece as usize][mv.from as usize]);
        let cont_2 = cont_2.map(|mv| &side.followup_move[mv.piece as usize][mv.from as usize]);
        let cont_4 = cont_4.map(|mv| &side.followup_move[mv.piece as usize][mv.from as usize]);
        HistSlice {
            nstm_threats,
            quiet: &side.quiet,
            noisy: &side.noisy,
            cont_1,
            cont_2,
            cont_4,
            pawn_corr: &side.pawn_corr[pos.pawn_hash() as usize],
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistSlice<'a> {
    nstm_threats: BitBoard,
    quiet: &'a [Butterfly<Entry>; 2],
    noisy: &'a [Butterfly<Entry>; 2],
    cont_1: Option<&'a PieceTo<Entry>>,
    cont_2: Option<&'a PieceTo<Entry>>,
    cont_4: Option<&'a PieceTo<Entry>>,
    pawn_corr: &'a Entry<i32>,
}

impl<'a> HistSlice<'a> {
    pub fn search_score(&self, mv: MoveData) -> i16 {
        if mv.capture {
            self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .get()
        } else {
            let mut score = self.quiet[self.nstm_threats.has(mv.from) as usize][mv.from as usize]
                [mv.to as usize]
                .get();
            if let Some(cont_1) = self.cont_1 {
                score += cont_1[mv.piece as usize][mv.to as usize].get();
            }
            score
        }
    }

    pub fn quiet_score(&self, mv: MoveData) -> i16 {
        assert!(!mv.capture);

        let mut score = self.quiet[self.nstm_threats.has(mv.from) as usize][mv.from as usize]
            [mv.to as usize]
            .get();
        if let Some(cont_1) = self.cont_1 {
            score += cont_1[mv.piece as usize][mv.to as usize].get();
        }
        if let Some(cont_2) = self.cont_2 {
            score += cont_2[mv.piece as usize][mv.to as usize].get();
        }
        if let Some(cont_4) = self.cont_4 {
            score += cont_4[mv.piece as usize][mv.to as usize].get();
        }
        score
    }

    pub fn noisy_score(&self, mv: MoveData) -> i16 {
        assert!(mv.capture);
        self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize].get()
    }

    fn update_any(&self, mv: MoveData, score: i16) {
        if mv.capture {
            self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .update(score);
        } else {
            self.quiet[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .update(score);
            if let Some(cont_1) = self.cont_1 {
                cont_1[mv.piece as usize][mv.to as usize].update(score);
            }
            if let Some(cont_2) = self.cont_2 {
                cont_2[mv.piece as usize][mv.to as usize].update(score);
            }
            if let Some(cont_4) = self.cont_4 {
                cont_4[mv.piece as usize][mv.to as usize].update(score);
            }
        }
    }

    pub fn update(
        &self,
        best_move: MoveData,
        quiets: &[MoveData],
        noisies: &[MoveData],
        score: i16,
    ) {
        self.update_any(best_move, score);
        for mv in noisies {
            self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .update(-score);
        }
        if best_move.capture {
            return;
        }
        for mv in quiets {
            self.quiet[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .update(-score);
        }
        if let Some(cont_1) = self.cont_1 {
            for mv in quiets {
                cont_1[mv.piece as usize][mv.to as usize].update(-score);
            }
        }
        if let Some(cont_2) = self.cont_2 {
            for mv in quiets {
                cont_2[mv.piece as usize][mv.to as usize].update(-score);
            }
        }
        if let Some(cont_4) = self.cont_4 {
            for mv in quiets {
                cont_4[mv.piece as usize][mv.to as usize].update(-score);
            }
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
        let new_value = *val + eval_diff as i32 * weight;
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
