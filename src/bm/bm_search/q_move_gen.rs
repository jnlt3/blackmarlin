use arrayvec::ArrayVec;
use cozy_chess::Move;

use crate::bm::bm_util::{history::History, position::Position};

use super::see::{calculate_see, move_value};

const MAX_MOVES: usize = 218;

#[derive(PartialEq, Eq)]
enum Phase {
    GenCaptures,
    GoodCaptures,
}

struct Capture {
    mv: Move,
    score: i16,
    see: i16,
}

impl Capture {
    pub fn new(mv: Move, score: i16, see: i16) -> Self {
        Self { mv, score, see }
    }
}

pub struct QSearchMoveGen {
    phase: Phase,
    captures: ArrayVec<Capture, MAX_MOVES>,
}

impl QSearchMoveGen {
    pub fn new() -> Self {
        Self {
            phase: Phase::GenCaptures,
            captures: ArrayVec::new(),
        }
    }

    pub fn next(&mut self, pos: &Position, hist: &History) -> Option<(Move, i16)> {
        if self.phase == Phase::GenCaptures {
            self.phase = Phase::GoodCaptures;
            let stm = pos.board().side_to_move();
            pos.board().generate_moves(|mut piece_moves| {
                piece_moves.to &= pos.board().colors(!stm);
                for mv in piece_moves {
                    let cap_hist = hist.get_capture(pos, mv);
                    let see = calculate_see(pos.board(), mv);
                    if cap_hist < 384 && see < 0 {
                        continue;
                    }
                    let score = cap_hist + move_value(pos.board(), mv) * 32;
                    self.captures.push(Capture::new(mv, score, see));
                }
                false
            });
        }
        if self.phase == Phase::GoodCaptures {
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
                return self
                    .captures
                    .swap_pop(index)
                    .map(|capture| (capture.mv, capture.see));
            }
        }
        None
    }
}
