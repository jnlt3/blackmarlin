use cozy_chess::{Board, Color, Move, Piece, Square};

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

    pub fn cutoff(&mut self, board: &Board, prev_move: Move, cutoff_move: Move, amt: u32) {
        if amt > 20 {
            return;
        }
        let color = board.side_to_move();
        let piece = board.piece_on(prev_move.to).unwrap_or(Piece::King);
        let to = prev_move.to;
        self.table[color as usize][piece as usize][to as usize] = Some(cutoff_move);
    }
}
