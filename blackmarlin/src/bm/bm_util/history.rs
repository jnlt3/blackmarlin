use std::alloc::Layout;
use std::cell::Cell;

use cozy_chess::{BitBoard, Move, Piece, Square};

use crate::bm::bm_runner::searcher::MoveData;

use super::position::Position;

pub const MAX_HIST: i16 = 512;

pub const CORR_HIST_GRAIN: i32 = 256;
pub const MAX_CORRECT: i32 = 32;

pub type Butterfly<T> = [[T; Square::NUM]; Square::NUM];
pub type PieceTo<T> = [[T; Square::NUM]; Piece::NUM];

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
        let decay = (score.abs() as i32 * value as i32 / MAX_HIST as i32) as i16;
        self.value.set(value + score - decay)
    }
}

fn hist_stat(amt: i16) -> i16 {
    (amt * 13).clamp(-MAX_HIST, MAX_HIST)
}

#[derive(Debug, Clone)]
struct SideHistory {
    quiet: [Butterfly<Entry>; 2],
    noisy: [Butterfly<Entry>; 2],
    counter_move: PieceTo<PieceTo<Entry>>,
    followup_move: PieceTo<PieceTo<Entry>>,
    pawn_corr: [Entry<i32>; u16::MAX as usize + 1],
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

    pub fn full(
        &self,
        pos: &Position,
        cont_1: Option<MoveData>,
        cont_2: Option<MoveData>,
        cont_4: Option<MoveData>,
    ) -> FullHist<'_> {
        let stm = pos.board().side_to_move();
        let side = &self.sides[stm as usize];
        let (_, nstm_threats) = pos.threats();

        let cont_1 = cont_1.map(|mv| &side.counter_move[mv.piece as usize][mv.to as usize]);
        let cont_2 = cont_2.map(|mv| &side.followup_move[mv.piece as usize][mv.to as usize]);
        let cont_4 = cont_4.map(|mv| &side.followup_move[mv.piece as usize][mv.to as usize]);
        FullHist {
            nstm_threats,
            quiet: &side.quiet,
            noisy: &side.noisy,
            cont_1,
            cont_2,
            cont_4,
            pawn_corr: &side.pawn_corr[pos.pawn_hash() as usize],
        }
    }

    pub fn noisy(&self, pos: &Position) -> NoisyHist<'_> {
        let stm = pos.board().side_to_move();
        let side = &self.sides[stm as usize];
        let (_, nstm_threats) = pos.threats();
        NoisyHist {
            nstm_threats,
            noisy: &side.noisy,
            pawn_corr: &side.pawn_corr[pos.pawn_hash() as usize],
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FullHist<'a> {
    nstm_threats: BitBoard,
    quiet: &'a [Butterfly<Entry>; 2],
    noisy: &'a [Butterfly<Entry>; 2],
    cont_1: Option<&'a PieceTo<Entry>>,
    cont_2: Option<&'a PieceTo<Entry>>,
    cont_4: Option<&'a PieceTo<Entry>>,
    pawn_corr: &'a Entry<i32>,
}

#[derive(Debug, Copy, Clone)]
pub struct NoisyHist<'a> {
    nstm_threats: BitBoard,
    noisy: &'a [Butterfly<Entry>; 2],
    pawn_corr: &'a Entry<i32>,
}

impl<'a> FullHist<'a> {
    pub fn correction(&self) -> i16 {
        (self.pawn_corr.get() / CORR_HIST_GRAIN) as i16
    }

    pub fn update_correction(&self, eval_diff: i16, depth: u32) {
        let value = self.pawn_corr.get();
        let weight = (depth * 8).min(128) as i32;
        let new_value = value + eval_diff as i32 * weight;
        let new_value = new_value.clamp(
            -MAX_CORRECT * CORR_HIST_GRAIN,
            MAX_CORRECT * CORR_HIST_GRAIN,
        );
        self.pawn_corr.value.set(new_value);
    }

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
            score / 2
        }
    }

    pub fn quiet_score(&self, mv: MoveData) -> i16 {
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

    pub fn noisy_score(&self, mv: Move) -> i16 {
        self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize].get()
    }

    pub fn update(
        &self,
        best_move: MoveData,
        quiets: &[MoveData],
        noisies: &[MoveData],
        score: i16,
    ) {
        if best_move.capture {
            self.noisy[self.nstm_threats.has(best_move.from) as usize][best_move.from as usize]
                [best_move.to as usize]
                .update(score);
        }
        for mv in noisies {
            self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .update(-score);
        }
        if best_move.capture {
            return;
        }
        self.quiet[self.nstm_threats.has(best_move.from) as usize][best_move.from as usize]
            [best_move.to as usize]
            .update(score);
        for mv in quiets {
            self.quiet[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize]
                .update(-score);
        }
        if let Some(cont_1) = self.cont_1 {
            cont_1[best_move.piece as usize][best_move.to as usize].update(score);
            for mv in quiets {
                cont_1[mv.piece as usize][mv.to as usize].update(-score);
            }
        }
        if let Some(cont_2) = self.cont_2 {
            cont_2[best_move.piece as usize][best_move.to as usize].update(score);
            for mv in quiets {
                cont_2[mv.piece as usize][mv.to as usize].update(-score);
            }
        }
        if let Some(cont_4) = self.cont_4 {
            cont_4[best_move.piece as usize][best_move.to as usize].update(score);
            for mv in quiets {
                cont_4[mv.piece as usize][mv.to as usize].update(-score);
            }
        }
    }
}

impl<'a> NoisyHist<'a> {
    pub fn noisy_score(&self, mv: Move) -> i16 {
        self.noisy[self.nstm_threats.has(mv.from) as usize][mv.from as usize][mv.to as usize].get()
    }

    pub fn correction(&self) -> i16 {
        (self.pawn_corr.get() / CORR_HIST_GRAIN) as i16
    }
}
