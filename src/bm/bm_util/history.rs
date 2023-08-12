use cozy_chess::{Color, Move, Piece, Square};

use crate::bm::bm_runner::ab_runner::SearchStack;

use super::position::Position;
use super::table_types::{new_butterfly_table, new_piece_to_table, Butterfly, PieceTo};

pub const MAX_HIST: i16 = 512;

fn hist_stat(amt: i16) -> i16 {
    (amt * 16).min(MAX_HIST)
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

pub const FOLLOWUP_TABLE_CNT: usize = 4;

/// Contains information calculated to index the history tables
#[derive(Copy, Clone)]
pub struct HistoryIndices {
    piece_to: [Option<(Piece, Square)>; FOLLOWUP_TABLE_CNT],
}

impl HistoryIndices {
    pub fn new(ss: &[SearchStack]) -> Self {
        let mut piece_to = [None; FOLLOWUP_TABLE_CNT];
        for index in 0..FOLLOWUP_TABLE_CNT {
            if index >= ss.len() {
                break;
            }
            piece_to[index] = ss[ss.len() - index - 1]
                .move_played
                .map(|mv| (mv.piece, mv.to));
        }
        Self { piece_to }
    }
}

#[derive(Debug, Clone)]
pub struct History {
    quiet: Box<[Butterfly<i16>; Color::NUM]>,
    capture: Box<[Butterfly<i16>; Color::NUM]>,
    followup_move: Box<[[PieceTo<PieceTo<i16>>; FOLLOWUP_TABLE_CNT]; Color::NUM]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            quiet: Box::new([new_butterfly_table(0); Color::NUM]),
            capture: Box::new([new_butterfly_table(0); Color::NUM]),
            followup_move: Box::new(
                [[new_piece_to_table(new_piece_to_table(0)); FOLLOWUP_TABLE_CNT]; Color::NUM],
            ),
        }
    }

    pub fn get_quiet(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        self.quiet[stm as usize][make_move.from as usize][make_move.to as usize]
    }

    fn get_quiet_mut(&mut self, pos: &Position, make_move: Move) -> &mut i16 {
        let stm = pos.board().side_to_move();
        &mut self.quiet[stm as usize][make_move.from as usize][make_move.to as usize]
    }

    pub fn get_capture(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        self.capture[stm as usize][make_move.from as usize][make_move.to as usize]
    }

    fn get_capture_mut(&mut self, pos: &Position, make_move: Move) -> &mut i16 {
        let stm = pos.board().side_to_move();
        &mut self.capture[stm as usize][make_move.from as usize][make_move.to as usize]
    }

    pub fn get_followup_move(
        &self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
        index: usize,
    ) -> Option<i16> {
        let (prev_piece, prev_to) = indices.piece_to[index]?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            self.followup_move[stm as usize][index][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    fn get_followup_move_mut(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
        index: usize,
    ) -> Option<&mut i16> {
        let (prev_piece, prev_to) = indices.piece_to[index]?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            &mut self.followup_move[stm as usize][index][prev_piece as usize][prev_to as usize]
                [current_piece as usize][make_move.to as usize],
        )
    }

    pub fn update_history(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
        quiets: &[Move],
        captures: &[Move],
        amt: i16,
    ) {
        let is_capture = pos
            .board()
            .colors(!pos.board().side_to_move())
            .has(make_move.to);
        if !is_capture {
            self.update_quiet(pos, indices, make_move, quiets, amt);
        } else {
            bonus(self.get_capture_mut(pos, make_move), amt);
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
        for index in 0..FOLLOWUP_TABLE_CNT {
            if let Some(followup_move_hist) =
                self.get_followup_move_mut(pos, indices, make_move, index)
            {
                bonus(followup_move_hist, amt);
                for &failed_move in fails {
                    let failed_hist = self
                        .get_followup_move_mut(pos, indices, failed_move, index)
                        .unwrap();
                    malus(failed_hist, amt);
                }
            }
        }
    }
}
