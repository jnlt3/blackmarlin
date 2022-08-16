use cozy_chess::{BitBoard, Color, Move, Piece, Square};

use super::position::Position;
use super::table_types::{
    new_butterfly_table, new_piece_to_table, new_threatened_table, Butterfly, PieceTo, Threatened,
};

pub const MAX_HIST: i16 = 512;

fn hist_stat(amt: i16) -> i16 {
    (amt * amt).min(MAX_HIST)
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
    counter_move: Option<(Piece, Square)>,
    nstm_threats: BitBoard,
}

impl HistoryIndices {
    pub fn new(pos: &Position, prev_move: Option<Move>, nstm_threats: BitBoard) -> Self {
        let counter_move = prev_move.map(|prev_move| {
            let piece = pos.board().piece_on(prev_move.to).unwrap_or(Piece::King);
            (piece, prev_move.to)
        });
        Self {
            counter_move,
            nstm_threats,
        }
    }
}

#[derive(Debug, Clone)]
pub struct History {
    quiet: Box<[Threatened<Butterfly<i16>>; Color::NUM]>,
    capture: Box<[Butterfly<i16>; Color::NUM]>,
    counter_move: Box<[PieceTo<PieceTo<i16>>; Color::NUM]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            quiet: Box::new([new_threatened_table(new_butterfly_table(0)); Color::NUM]),
            capture: Box::new([new_butterfly_table(0); Color::NUM]),
            counter_move: Box::new([new_piece_to_table(new_piece_to_table(0)); Color::NUM]),
        }
    }

    pub fn get_quiet(&self, pos: &Position, indices: &HistoryIndices, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        let is_threatened = indices.nstm_threats.has(make_move.from) as usize;
        self.quiet[stm as usize][is_threatened][make_move.from as usize][make_move.to as usize]
    }

    fn get_quiet_mut(
        &mut self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> &mut i16 {
        let stm = pos.board().side_to_move();
        let is_threatened = indices.nstm_threats.has(make_move.from) as usize;
        &mut self.quiet[stm as usize][is_threatened][make_move.from as usize][make_move.to as usize]
    }

    pub fn get_capture(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        self.capture[stm as usize][make_move.from as usize][make_move.to as usize]
    }

    fn get_capture_mut(&mut self, pos: &Position, make_move: Move) -> &mut i16 {
        let stm = pos.board().side_to_move();
        &mut self.capture[stm as usize][make_move.from as usize][make_move.to as usize]
    }

    pub fn get_counter_move(
        &self,
        pos: &Position,
        indices: &HistoryIndices,
        make_move: Move,
    ) -> Option<i16> {
        let (prev_piece, prev_to) = indices.counter_move?;
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
        let (prev_piece, prev_to) = indices.counter_move?;
        let stm = pos.board().side_to_move();
        let current_piece = pos.board().piece_on(make_move.from).unwrap();
        Some(
            &mut self.counter_move[stm as usize][prev_piece as usize][prev_to as usize]
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
        bonus(self.get_quiet_mut(pos, indices, make_move), amt);
        for &failed_move in fails {
            malus(self.get_quiet_mut(pos, indices, failed_move), amt);
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
    }
}
