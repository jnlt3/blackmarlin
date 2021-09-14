use chess::{Color, Piece, Square};
use std::sync::atomic::{AtomicU32, Ordering};


const PIECE_COUNT: usize = 12;
const MAX: u32 = 32768;

#[derive(Debug, Copy, Clone)]
pub struct PieceTo {
    pub piece: Piece,
    pub to: Square,
}

type CmhtU32 = [[[u32; 64]; PIECE_COUNT]; 64];
type CmhtAtomicU32 = [[[AtomicU32; 64]; PIECE_COUNT]; 64];

#[derive(Debug)]
pub struct CMoveHistoryTable {
    table: Vec<CmhtAtomicU32>,
}

impl CMoveHistoryTable {
    pub fn new() -> Self {
        Self {
            table: (0..PIECE_COUNT)
                .into_iter()
                .map(|_| unsafe {
                    std::mem::transmute::<CmhtU32, CmhtAtomicU32>([[[0u32; 64]; PIECE_COUNT]; 64])
                })
                .collect::<Vec<CmhtAtomicU32>>(),
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

    pub fn get(&self, color: Color, piece: Piece, to: Square, c_piece: Piece, c_to: Square) -> u32 {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        let c_piece_index = Self::piece_index(color, c_piece);
        let c_sqr_index = c_to.to_index();
        self.table[piece_index][sqr_index][c_piece_index][c_sqr_index].load(Ordering::SeqCst)
    }

    pub fn add(
        &self,
        color: Color,
        piece: Piece,
        to: Square,
        c_piece: Piece,
        c_to: Square,
        amt: u32,
    ) {
        let piece_index = Self::piece_index(color, piece);
        let sqr_index = to.to_index();
        let c_piece_index = Self::piece_index(color, c_piece);
        let c_sqr_index = c_to.to_index();
        let current_value = &self.table[piece_index][sqr_index][c_piece_index][c_sqr_index];
        current_value.fetch_add(amt, Ordering::SeqCst);
        current_value.fetch_min(MAX, Ordering::SeqCst);
    }

    pub fn for_all<F: Fn(u32) -> u32>(&self, func: F) {
        for x in &*self.table {
            for y in x {
                for z in y {
                    for t in z {
                        t.store(func(t.load(Ordering::SeqCst)), Ordering::SeqCst);
                    }
                }
            }
        }
    }
}
