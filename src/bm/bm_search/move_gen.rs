use cozy_chess::Move;

use super::move_entry::{MoveEntry, MoveEntryIterator};
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

    fn is_good_capture<F: FnOnce(Move) -> bool>(&mut self, f: F) -> bool {
        match self.good_capture {
            Some(good_capture) => good_capture,
            None => {
                let good_capture = f(self.mv);
                self.good_capture = Some(good_capture);
                good_capture
            }
        }
    }
}

pub struct OrderedMoveGen {
    phase: Phase,

    pv_move: Option<Move>,

    killers: MoveEntry<2>,
    killer_iter: MoveEntryIterator<2>,

    piece_moves: ArrayVec<PieceMoves, 18>,

    quiets: ArrayVec<Quiet, MAX_MOVES>,
    captures: ArrayVec<Capture, MAX_MOVES>,
}

impl OrderedMoveGen {
    pub fn new(board: &Board, pv_move: Option<Move>, killers: MoveEntry<2>) -> Self {
        Self {
            phase: Phase::PvMove,
            pv_move: pv_move.filter(|&mv| board.is_legal(mv)),
            killer_iter: killers.into_iter(),
            killers,
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
                    if self.killers.into_iter().any(|killer| mv == killer) {
                        continue;
                    }
                    let score = hist.get_capture(pos, mv) + move_value(pos.board(), mv) * 32;
                    self.captures.push(Capture::new(mv, score));
                }
            }
        }
        if self.phase == Phase::GoodCaptures {
            let mut best_capture = None;
            for (index, capture) in self.captures.iter_mut().enumerate() {
                if !capture.is_good_capture(|mv| compare_see(pos.board(), mv, 0)) {
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
            while let Some(killer) = self.killer_iter.next() {
                if !pos.board().is_legal(killer) {
                    continue;
                }
                return Some(killer);
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
                    if self.killers.into_iter().any(|killer| mv == killer) {
                        continue;
                    }

                    let score = match mv.promotion {
                        Some(Piece::Queen) => i16::MAX,
                        Some(_) => i16::MIN,
                        _ => {
                            let quiet_hist = hist.get_quiet(pos, mv);
                            let counter_move_hist = hist
                                .get_counter_move(pos, hist_indices, mv)
                                .unwrap_or_default();
                            quiet_hist + counter_move_hist
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

type LazySee = Option<i16>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QSearchGenType {
    CalcCaptures,
    Captures,
}

pub struct QuiescenceSearchMoveGen {
    gen_type: QSearchGenType,
    queue: ArrayVec<(Move, i16, LazySee), MAX_MOVES>,
}

impl QuiescenceSearchMoveGen {
    pub fn new() -> Self {
        Self {
            gen_type: QSearchGenType::CalcCaptures,
            queue: ArrayVec::new(),
        }
    }

    pub fn next(&mut self, pos: &Position, hist: &History) -> Option<(Move, i16)> {
        let board = pos.board();
        if self.gen_type == QSearchGenType::CalcCaptures {
            board.generate_moves(|mut piece_moves| {
                piece_moves.to &= board.colors(!board.side_to_move());
                for make_move in piece_moves {
                    let expected_gain =
                        hist.get_capture(pos, make_move) + move_value(board, make_move) * 32;
                    self.queue.push((make_move, expected_gain, None));
                }
                false
            });
            self.gen_type = QSearchGenType::Captures;
        }
        let mut max = 0;
        let mut best_index = None;
        for (index, (make_move, score, see)) in self.queue.iter_mut().enumerate() {
            if best_index.is_none() || *score > max {
                let see_score = see.unwrap_or_else(|| calculate_see(board, *make_move));
                *see = Some(see_score);
                if see_score < 0 {
                    continue;
                }
                max = *score;
                best_index = Some(index);
            }
        }
        if let Some(index) = best_index {
            self.queue
                .swap_pop(index)
                .map(|(make_move, _, see)| (make_move, see.unwrap()))
        } else {
            None
        }
    }
}
