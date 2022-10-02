use crate::bm::bm_runner::ab_runner::MoveData;

use super::table_types::{new_piece_to_table, PieceTo};
use cozy_chess::Move;

#[derive(Debug, Clone)]
pub struct MoveTable {
    table: PieceTo<Option<Move>>,
}

impl MoveTable {
    pub fn new() -> Self {
        Self {
            table: new_piece_to_table(None),
        }
    }

    pub fn update(&mut self, prev_move: MoveData, refutation: Move) {
        self.table[prev_move.piece as usize][prev_move.to as usize] = Some(refutation);
    }

    pub fn get(&self, prev_move: MoveData) -> Option<Move> {
        self.table[prev_move.piece as usize][prev_move.to as usize]
    }
}
