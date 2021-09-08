use chess::{Color, Piece, Square};
use std::sync::atomic::{AtomicU32, Ordering};


const PIECE_COUNT: usize = 12;
const MAX: u32 = 32768;

#[derive(Debug)]
pub struct HistoryTable {
    table: Box<[[AtomicU32; 64]; PIECE_COUNT]>,
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: unsafe {
                Box::new(std::mem::transmute::<
                    [[u32; 64]; PIECE_COUNT],
                    [[AtomicU32; 64]; PIECE_COUNT],
                >([[0u32; 64]; PIECE_COUNT]))
            },
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

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> u32 {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        self.table[piece_index][sqr_index].load(Ordering::Relaxed)
    }

    pub fn add(&self, color: Color, piece: Piece, to: Square, amt: u32) {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        self.table[piece_index][sqr_index].fetch_add(amt, Ordering::Relaxed);
        self.table[piece_index][sqr_index].fetch_min(MAX, Ordering::Relaxed);
    }

    pub fn for_all<F: Fn(u32) -> u32>(&self, func: F) {
        for piece_table in self.table.iter() {
            for sq in piece_table {
                sq.store(func(sq.load(Ordering::Relaxed)), Ordering::Relaxed)
            }
        }
    }
}
