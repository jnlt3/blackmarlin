use cozy_chess::Move;

use super::move_entry::MoveEntry;
use super::see::{compare_see, move_value};
use crate::bm::bm_runner::searcher::MoveData;
use crate::bm::bm_util::history::FullHist;
use crate::bm::bm_util::history::NoisyHist;
use crate::bm::bm_util::position::Position;
use arrayvec::ArrayVec;
use cozy_chess::{Piece, PieceMoves};

const MAX_MOVES: usize = 218;

#[derive(PartialEq, Eq, Copy, Debug, Clone, PartialOrd, Ord)]
pub enum Phase {
    PvMove,
    GenPieceMoves,
    GenCaptures,
    /// Generated move has a SEE value >= 0
    GoodCaptures,
    /// Generated move is a killer move
    Killers,
    GenQuiets,
    /// Generated move is a non capture
    Quiets,
    /// Generated move has a SEE value < 0
    BadCaptures,
}

struct ScoredMove {
    mv: Move,
    score: i16,
}

impl ScoredMove {
    pub fn new(mv: Move, score: i16) -> Self {
        Self { mv, score }
    }
}
pub struct OrderedMoveGen<'a> {
    phase: Phase,

    pv_move: Option<Move>,

    killers: MoveEntry,
    killer_index: usize,

    history: FullHist<'a>,

    piece_moves: ArrayVec<PieceMoves, 18>,

    quiets: ArrayVec<ScoredMove, MAX_MOVES>,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
    bad_captures: ArrayVec<ScoredMove, MAX_MOVES>,
    bad_capture_idx: usize,
}

fn select_highest(array: &[ScoredMove]) -> Option<usize> {
    if array.is_empty() {
        return None;
    }
    let mut best: Option<(i16, usize)> = None;
    for (index, mv) in array.iter().enumerate() {
        if let Some((best_score, _)) = best {
            if mv.score <= best_score {
                continue;
            }
        }
        best = Some((mv.score, index));
    }
    best.map(|(_, index)| index)
}

impl<'a> OrderedMoveGen<'a> {
    /// Expects legal PV move
    /// Killers are verified for legality in [next](OrderedMoveGen::next)
    pub fn new(pv_move: Option<Move>, killers: MoveEntry, history: FullHist<'a>) -> Self {
        Self {
            phase: Phase::PvMove,
            pv_move,
            history,
            killers,
            killer_index: 0,
            piece_moves: ArrayVec::new(),
            quiets: ArrayVec::new(),
            captures: ArrayVec::new(),
            bad_captures: ArrayVec::new(),
            bad_capture_idx: 0,
        }
    }

    /// Returns what phase of move generation the last generated came from
    /// with the exception of TT move.
    ///
    /// The proper way to check if the generated move is a TT move is a direct comparison of the moves
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Skips all quiet generation only if the last generated move is a quiet move that is not from the TT  
    pub fn skip_quiets(&mut self) {
        self.phase = match self.phase {
            Phase::Killers | Phase::GenQuiets | Phase::Quiets => Phase::BadCaptures,
            _ => return,
        }
    }

    /// Generate a legal move that hasn't been given before or
    ///  return [None](Option::None) if no legal moves are left
    ///
    /// Performance Notes:
    /// - Move generation and capture scoring is done after TT move is returned
    /// - When generating good captures, each call is at least one [see](super::see::compare_see) call
    /// - Quiet move lists are generated and scored after killer moves are returned
    pub fn next(&mut self, pos: &Position) -> Option<Move> {
        if self.phase == Phase::PvMove {
            self.phase = Phase::GenPieceMoves;
            if self.pv_move.is_some() {
                return self.pv_move;
            }
        }
        if self.phase == Phase::GenPieceMoves {
            self.phase = Phase::GenCaptures;
            pos.board().generate_moves(|piece_moves| {
                self.piece_moves.push(piece_moves);
                false
            });
        }
        if self.phase == Phase::GenCaptures {
            self.phase = Phase::GoodCaptures;
            let stm = pos.board().side_to_move();
            for mut piece_moves in self.piece_moves.iter().copied() {
                piece_moves.to &= pos.board().colors(!stm);
                for mv in piece_moves {
                    if Some(mv) == self.pv_move {
                        continue;
                    }
                    if let Some(index) = self.killers.index_of(mv) {
                        self.killers.remove(index);
                    }
                    let score = self.history.noisy_score(mv) + move_value(pos.board(), mv) * 32;
                    self.captures.push(ScoredMove::new(mv, score))
                }
            }
        }
        if self.phase == Phase::GoodCaptures {
            while let Some(index) = select_highest(&self.captures) {
                let capture = self.captures.swap_remove(index);
                if !compare_see(pos.board(), capture.mv, 0) {
                    self.bad_captures.push(capture);
                    continue;
                }
                return Some(capture.mv);
            }
            self.phase = Phase::Killers;
        }
        if self.phase == Phase::Killers {
            while self.killer_index < 2 {
                let killer = self.killers.get(self.killer_index);
                self.killer_index += 1;
                if let Some(killer) = killer {
                    if Some(killer) == self.pv_move {
                        continue;
                    }
                    if !pos.board().is_legal(killer) {
                        continue;
                    }
                    return Some(killer);
                }
            }
            self.phase = Phase::GenQuiets;
        }
        if self.phase == Phase::GenQuiets {
            self.phase = Phase::Quiets;
            let stm = pos.board().side_to_move();
            for mut piece_moves in self.piece_moves.iter().copied() {
                piece_moves.to &= !pos.board().colors(!stm);
                for mv in piece_moves {
                    if Some(mv) == self.pv_move {
                        continue;
                    }
                    if self.killers.contains(mv) {
                        continue;
                    }

                    let score = match mv.promotion {
                        Some(Piece::Queen) => i16::MAX,
                        Some(_) => i16::MIN,
                        None => self
                            .history
                            .quiet_score(MoveData::from_move(pos.board(), mv)),
                    };
                    self.quiets.push(ScoredMove::new(mv, score));
                }
            }
        }
        if self.phase == Phase::Quiets {
            if let Some(index) = select_highest(&self.quiets) {
                return self.quiets.swap_pop(index).map(|quiet| quiet.mv);
            }
            self.phase = Phase::BadCaptures;
        }
        // Bad captures are already implicitly sorted from best to worst
        let out = self
            .bad_captures
            .get(self.bad_capture_idx)
            .map(|capture| capture.mv);
        self.bad_capture_idx += 1;
        out
    }
}

#[derive(PartialEq, Eq)]
enum QPhase {
    GenCaptures,
    GoodCaptures,
}

pub struct QSearchMoveGen<'a> {
    phase: QPhase,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
    history: NoisyHist<'a>,
}

impl<'a> QSearchMoveGen<'a> {
    pub fn new(history: NoisyHist<'a>) -> Self {
        Self {
            phase: QPhase::GenCaptures,
            captures: ArrayVec::new(),
            history,
        }
    }

    /// Generate a legal capture that hasn't been generated before primarily ordered
    /// by captured pieces value and then capture history
    /// - En-passant is ignored
    pub fn next(&mut self, pos: &Position) -> Option<Move> {
        if self.phase == QPhase::GenCaptures {
            self.phase = QPhase::GoodCaptures;
            let stm = pos.board().side_to_move();
            pos.board().generate_moves(|mut piece_moves| {
                piece_moves.to &= pos.board().colors(!stm);
                for mv in piece_moves {
                    let score = self.history.noisy_score(mv) + move_value(pos.board(), mv) * 32;
                    self.captures.push(ScoredMove::new(mv, score));
                }
                false
            });
        }
        if self.phase == QPhase::GoodCaptures {
            while let Some(index) = select_highest(&self.captures) {
                let capture = self.captures.swap_remove(index).mv;
                return Some(capture);
            }
        }
        None
    }
}
