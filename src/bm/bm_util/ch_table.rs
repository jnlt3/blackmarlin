use chess::{Color, Piece, Square};
use std::sync::atomic::{AtomicU32, Ordering};

const PIECE_COUNT: usize = 12;

type ChtU32 = [[[u32; PIECE_COUNT]; 64]; PIECE_COUNT];
type ChtAtomicU32 = [[[AtomicU32; PIECE_COUNT]; 64]; PIECE_COUNT];

#[derive(Debug)]
pub struct CaptureHistoryTable {
    table: Box<ChtAtomicU32>,
}

impl CaptureHistoryTable {
    pub fn new() -> Self {
        Self {
            table: unsafe {
                Box::new(std::mem::transmute::<ChtU32, ChtAtomicU32>(
                    [[[0u32; PIECE_COUNT]; 64]; PIECE_COUNT],
                ))
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

    pub fn get(&self, color: Color, piece: Piece, to: Square, captured: Piece) -> u32 {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        self.table[piece_index][sqr_index][captured.to_index()].load(Ordering::SeqCst)
    }

    pub fn add(&self, color: Color, piece: Piece, to: Square, captured: Piece, amt: u32) {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        let current_value = &self.table[piece_index][sqr_index][captured.to_index()];
        current_value.fetch_add(amt, Ordering::SeqCst);
    }

    pub fn for_all<F: Fn(u32) -> u32>(&self, func: F) {
        for piece_table in self.table.iter() {
            for sq in piece_table {
                for capture in sq {
                    capture.store(func(capture.load(Ordering::SeqCst)), Ordering::SeqCst)
                }
            }
        }
    }
}
