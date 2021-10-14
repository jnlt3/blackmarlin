use chess::{Board, ChessMove, Color, Piece, Square};
use std::sync::atomic::{AtomicI16, Ordering};


const MAX_VALUE: i16 = 512;
const PIECE_COUNT: usize = 12;

#[derive(Debug)]
pub struct HistoryTable {
    table: Box<[[AtomicI16; 64]; PIECE_COUNT]>,
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: unsafe {
                Box::new(std::mem::transmute([[0_i16; 64]; PIECE_COUNT]))
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

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> i16 {
        let piece_index = Self::piece_index(color, piece);
        let to_index = to.to_index();
        self.table[piece_index][to_index].load(Ordering::SeqCst)
    }

    pub fn cutoff(&self, board: &Board, make_move: ChessMove, quiets: &[ChessMove], amt: u32) {
        let piece = board.piece_on(make_move.get_source()).unwrap();
        let piece_index = Self::piece_index(board.side_to_move(), piece);
        let to_index = make_move.get_dest().to_index();
        
        let value = self.table[piece_index][to_index].load(Ordering::SeqCst);
        let change = (amt * amt) as i16;
        let decay = change * value / MAX_VALUE;

        let increment = change - decay;

        self.table[piece_index][to_index].fetch_add(increment, Ordering::SeqCst);

        for &quiet in quiets {
            let piece = board.piece_on(quiet.get_source()).unwrap();
            let piece_index = Self::piece_index(board.side_to_move(), piece);
            let to_index = quiet.get_dest().to_index();
            let value = self.table[piece_index][to_index].load(Ordering::SeqCst);
            let decay = change * value / MAX_VALUE;
            let decrement = change + decay;
    
            self.table[piece_index][to_index].fetch_sub(increment, Ordering::SeqCst);
        }
    }

    pub fn for_all<F: Fn(i16) -> i16>(&self, func: F) {
        for piece_table in self.table.iter() {
            for sq in piece_table {
                sq.store(func(sq.load(Ordering::SeqCst)), Ordering::SeqCst)
            }
        }
    }
}
