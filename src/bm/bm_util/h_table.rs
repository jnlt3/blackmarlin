use chess::{Board, ChessMove, Color, Piece, Square};

const MAX_VALUE: i16 = 512;
const SQUARE_COUNT: usize = 64;
const PIECE_COUNT: usize = 12;

#[derive(Debug, Clone)]
pub struct HistoryTable {
    table: Box<[[i16; SQUARE_COUNT]; PIECE_COUNT]>,
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: Box::new([[0_i16; SQUARE_COUNT]; PIECE_COUNT]),
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
        self.table[piece_index][to_index]
    }

    pub fn cutoff(&mut self, board: &Board, make_move: ChessMove, quiets: &[ChessMove], amt: u32) {
        let piece = board.piece_on(make_move.get_source()).unwrap();
        let piece_index = Self::piece_index(board.side_to_move(), piece);
        let to_index = make_move.get_dest().to_index();

        let value = self.table[piece_index][to_index];
        let change = (amt * amt) as i16;
        let decay = change * value / MAX_VALUE;

        let increment = change - decay;

        self.table[piece_index][to_index] += increment;

        for &quiet in quiets {
            let piece = board.piece_on(quiet.get_source()).unwrap();
            let piece_index = Self::piece_index(board.side_to_move(), piece);
            let to_index = quiet.get_dest().to_index();
            let value = self.table[piece_index][to_index];
            let decay = change * value / MAX_VALUE;
            let decrement = change + decay;

            self.table[piece_index][to_index] -= decrement;
        }
    }

    pub fn for_all<F: Fn(i16) -> i16>(&mut self, func: F) {
        for piece_table in self.table.iter_mut() {
            for sq in piece_table {
                *sq = func(*sq);
            }
        }
    }
}


/*
struct AtomicMove {
    make_move: ChessMove,
    fill: u8,
}

pub struct CounterMoveHistory {
    table: Box<[[]]>
}
*/