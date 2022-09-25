use cozy_chess::{Board, Color, Move, Piece, Square};

use crate::bm::bm_runner::ab_runner::MoveData;

use super::table_types::{new_piece_to_table, PieceTo};

#[derive(Debug, Copy, Clone)]
struct Entry {
    make_move: Move,
    depth: u8,
}

#[derive(Debug, Clone)]
pub struct CounterMoveTable {
    table: Box<[PieceTo<Option<Entry>>; Color::NUM]>,
}

impl CounterMoveTable {
    pub fn new() -> Self {
        Self {
            table: Box::new([new_piece_to_table(None); Color::NUM]),
        }
    }

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> Option<Move> {
        self.table[color as usize][piece as usize][to as usize].map(|entry| entry.make_move)
    }

    pub fn set(&mut self, board: &Board, prev_move: MoveData, cutoff_move: Move, depth: u32) {
        let color = board.side_to_move();
        let entry = Entry {
            make_move: cutoff_move,
            depth: depth as u8,
        };
        let old_entry =
            &mut self.table[color as usize][prev_move.piece as usize][prev_move.to as usize];
        if let Some(old_entry) = old_entry {
            if entry.depth * 3 >= old_entry.depth {
                *old_entry = entry;
            }
            return;
        }
        *old_entry = Some(entry);
    }
}
