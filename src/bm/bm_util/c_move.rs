use chess::{ChessMove, Color, Piece, Square};

use std::sync::Mutex;

const PIECE_COUNT: usize = 12;

#[derive(Debug)]
pub struct CounterMoveTable {
    table: Mutex<[[Option<ChessMove>; 64]; PIECE_COUNT]>,
}

impl CounterMoveTable {
    pub fn new() -> Self {
        Self {
            table: Mutex::new([[None; 64]; PIECE_COUNT]),
        }
    }

    fn piece_index(color: Color, piece: Piece) -> usize {
        let color_offset = match color {
            Color::White => 0,
            Color::Black => PIECE_COUNT / 2,
        };
        let piece_index = piece.to_index();
        color_offset + piece_index
    }

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> Option<ChessMove> {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        let table = &*self.table.lock().unwrap();
        table[piece_index][sqr_index]
    }

    pub fn add(&self, color: Color, piece: Piece, to: Square, make_move: ChessMove) {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        let table = &mut *self.table.lock().unwrap();
        table[piece_index][sqr_index] = Some(make_move);
    }

    pub fn clear(&self) {
        let table = &mut *self.table.lock().unwrap();
        for piece_table in table {
            for sq in piece_table {
                *sq = None;
            }
        }
    }
}
