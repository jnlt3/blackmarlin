use cozy_chess::Move;

use super::move_entry::MoveEntry;
use super::see::{calculate_see, compare_see, move_value};
use crate::bm::bm_util::history::History;
use crate::bm::bm_util::history::HistoryIndices;
use crate::bm::bm_util::position::Position;
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
}

impl Capture {
    pub fn new(mv: Move, score: i16) -> Self {
        Self { mv, score }
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
    bad_captures: ArrayVec<Capture, MAX_MOVES>,
}

fn select_highest<T, U: Ord, S: Fn(&T) -> U>(array: &[T], score: S) -> Option<usize> {
    if array.is_empty() {
        return None;
    }
    let mut best: Option<(U, usize)> = None;
    for (index, mv) in array.iter().enumerate() {
        let score = score(mv);
        if let Some((best_score, _)) = &best {
            if &score <= best_score {
                continue;
            }
        }
        best = Some((score, index));
    }
    best.map(|(_, index)| index)
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
            bad_captures: ArrayVec::new(),
        }
    }

    pub fn skip_quiets(&mut self) {
        self.phase = match self.phase {
            Phase::Killers | Phase::GenQuiets | Phase::Quiets => Phase::BadCaptures,
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
                    self.captures.push(Capture::new(mv, score))
                }
            }
        }
        if self.phase == Phase::GoodCaptures {
            while let Some(index) = select_highest(&self.captures, |capture| capture.score) {
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
                        None => {
                            let quiet_hist = hist.get_quiet(pos, mv);
                            let counter_move_hist = hist
                                .get_counter_move(pos, hist_indices, mv)
                                .unwrap_or_default();
                            let followup_move_hist = hist
                                .get_followup_move(pos, hist_indices, mv)
                                .unwrap_or_default();
                            let pawn_hist = hist.get_pawn(pos, mv);
                            quiet_hist + counter_move_hist + followup_move_hist + pawn_hist
                        }
                    };
                    self.quiets.push(Quiet::new(mv, score));
                }
            }
        }
        if self.phase == Phase::Quiets {
            if let Some(index) = select_highest(&self.quiets, |quiet| quiet.score) {
                return self.quiets.swap_pop(index).map(|quiet| quiet.mv);
            }
            self.phase = Phase::BadCaptures;
        }
        if self.phase == Phase::BadCaptures {
            if let Some(index) = select_highest(&self.bad_captures, |capture| capture.score) {
                return self.bad_captures.swap_pop(index).map(|capture| capture.mv);
            }
        }
        None
    }
}

#[derive(PartialEq, Eq)]
enum QPhase {
    GenCaptures,
    GoodCaptures,
}

pub struct QSearchMoveGen {
    phase: QPhase,
    captures: ArrayVec<Capture, MAX_MOVES>,
}

impl QSearchMoveGen {
    pub fn new() -> Self {
        Self {
            phase: QPhase::GenCaptures,
            captures: ArrayVec::new(),
        }
    }

    pub fn next(&mut self, pos: &Position, hist: &History) -> Option<(Move, i16)> {
        if self.phase == QPhase::GenCaptures {
            self.phase = QPhase::GoodCaptures;
            let stm = pos.board().side_to_move();
            pos.board().generate_moves(|mut piece_moves| {
                piece_moves.to &= pos.board().colors(!stm);
                for mv in piece_moves {
                    let score = hist.get_capture(pos, mv) + move_value(pos.board(), mv) * 32;
                    self.captures.push(Capture::new(mv, score));
                }
                false
            });
        }
        if self.phase == QPhase::GoodCaptures {
            while let Some(index) = select_highest(&self.captures, |capture| capture.score) {
                let capture = self.captures.swap_remove(index).mv;
                let see = calculate_see(pos.board(), capture);
                if see < 0 {
                    continue;
                }
                return Some((capture, see));
            }
        }
        None
    }
}
