use cozy_chess::{Board, Color, Move, Piece, Square};

use crate::bm::bm_runner::ab_runner::MoveData;

use super::table_types::{new_piece_to_table, PieceTo};

#[derive(Debug, Clone)]
pub struct CounterMoveTable {
    table: Box<[PieceTo<Option<Move>>; Color::NUM]>,
}

impl CounterMoveTable {
    pub fn new() -> Self {
        Self {
            table: Box::new([new_piece_to_table(None); Color::NUM]),
        }
    }

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> Option<Move> {
        self.table[color as usize][piece as usize][to as usize]
    }

    pub fn cutoff(&mut self, board: &Board, prev_move: MoveData, cutoff_move: Move) {
        let color = board.side_to_move();
        self.table[color as usize][prev_move.piece as usize][prev_move.to as usize] =
            Some(cutoff_move);
    }
}
