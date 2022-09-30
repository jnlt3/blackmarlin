use cozy_chess::Move;

use super::move_entry::MoveEntry;
use super::see::{compare_see, move_value};
use crate::bm::bm_util::history::History;
use crate::bm::bm_util::history::HistoryIndices;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::threats::into_threat;
use arrayvec::ArrayVec;
use cozy_chess::{Board, Piece, PieceMoves};

const MAX_MOVES: usize = 218;

#[derive(PartialEq, Eq)]
enum Phase {
    PvMove,
    GenPieceMoves,
    GenCaptures,
    GoodCaptures,
    Killers,
    GenQuiets,
    Quiets,
    BadCaptures,
}

struct Quiet {
    mv: Move,
    score: i16,
}

impl Quiet {
    pub fn new(mv: Move, score: i16) -> Self {
        Self { mv, score }
    }
}

struct Capture {
    mv: Move,
    score: i16,
    good_capture: Option<bool>,
}

impl Capture {
    pub fn new(mv: Move, score: i16) -> Self {
        Self {
            mv,
            score,
            good_capture: None,
        }
    }

    fn is_good_capture(&mut self, board: &Board) -> bool {
        match self.good_capture {
            Some(good_capture) => good_capture,
            None => {
                let good_capture = compare_see(board, self.mv, 0);
                self.good_capture = Some(good_capture);
                good_capture
            }
        }
    }
}

pub struct OrderedMoveGen {
    phase: Phase,

    pv_move: Option<Move>,

    killers: MoveEntry,
    killer_index: usize,

    piece_moves: ArrayVec<PieceMoves, 18>,

    quiets: ArrayVec<Quiet, MAX_MOVES>,
    captures: ArrayVec<Capture, MAX_MOVES>,
}

impl OrderedMoveGen {
    pub fn new(board: &Board, pv_move: Option<Move>, killers: MoveEntry) -> Self {
        Self {
            phase: Phase::PvMove,
            pv_move: pv_move.filter(|&mv| board.is_legal(mv)),
            killers,
            killer_index: 0,
            piece_moves: ArrayVec::new(),
            quiets: ArrayVec::new(),
            captures: ArrayVec::new(),
        }
    }

    pub fn skip_quiets(&mut self) {
        self.phase = match self.phase {
            Phase::GoodCaptures | Phase::Killers | Phase::GenQuiets | Phase::Quiets => {
                Phase::BadCaptures
            }
            _ => return,
        }
    }

    pub fn next(
        &mut self,
        pos: &Position,
        hist: &History,
        hist_indices: &HistoryIndices,
    ) -> Option<Move> {
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
                    let score = hist.get_capture(pos, mv) + move_value(pos.board(), mv) * 32;
                    self.captures.push(Capture::new(mv, score));
                }
            }
        }
        if self.phase == Phase::GoodCaptures {
            let mut best_capture = None;
            for (index, capture) in self.captures.iter_mut().enumerate() {
                if !capture.is_good_capture(pos.board()) {
                    continue;
                }
                if let Some((score, _)) = best_capture {
                    if capture.score <= score {
                        continue;
                    }
                }
                best_capture = Some((capture.score, index));
            }
            if let Some((_, index)) = best_capture {
                return self.captures.swap_pop(index).map(|capture| capture.mv);
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
                let piece = piece_moves.piece;
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
                        None => {
                            let quiet_hist = hist.get_quiet(pos, mv);
                            let counter_move_hist = hist
                                .get_counter_move(pos, hist_indices, mv)
                                .unwrap_or_default();
                            let into_threat = match into_threat(pos.board(), piece, mv.to) {
                                true => -256,
                                false => 0,
                            };
                            quiet_hist + counter_move_hist + into_threat
                        }
                    };
                    self.quiets.push(Quiet::new(mv, score));
                }
            }
        }
        if self.phase == Phase::Quiets {
            let mut best_quiet = None;
            for (index, quiet) in self.quiets.iter_mut().enumerate() {
                if let Some((score, _)) = best_quiet {
                    if quiet.score <= score {
                        continue;
                    }
                }
                best_quiet = Some((quiet.score, index));
            }
            if let Some((_, index)) = best_quiet {
                return self.quiets.swap_pop(index).map(|quiet| quiet.mv);
            }
            self.phase = Phase::BadCaptures;
        }
        if self.phase == Phase::BadCaptures {
            let mut best_capture = None;
            for (index, capture) in self.captures.iter_mut().enumerate() {
                if let Some((score, _)) = best_capture {
                    if capture.score <= score {
                        continue;
                    }
                }
                best_capture = Some((capture.score, index));
            }
            if let Some((_, index)) = best_capture {
                return self.captures.swap_pop(index).map(|capture| capture.mv);
            }
        }
        None
    }
}
