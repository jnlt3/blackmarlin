use cozy_chess::{Board, Color, Move, Piece, Square};

const SQUARE_COUNT: usize = 64;
const PIECE_COUNT: usize = 12;

#[derive(Debug, Clone)]
pub struct CounterMoveTable {
    table: Box<[[Option<Move>; SQUARE_COUNT]; PIECE_COUNT]>,
}

impl CounterMoveTable {
    pub fn new() -> Self {
        Self {
            table: Box::new([[None; SQUARE_COUNT]; PIECE_COUNT]),
        }
    }

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> Option<Move> {
        let piece_index = color as usize * 6 + piece as usize;
        let to_index = to as usize;
        self.table[piece_index][to_index]
    }

    pub fn cutoff(&mut self, board: &Board, prev_move: Move, cutoff_move: Move, amt: u32) {
        if amt > 20 {
            return;
        }
        let piece = board.piece_on(prev_move.to).unwrap_or(Piece::King);
        let piece_index = board.side_to_move() as usize * 6 + piece as usize;
        let to_index = prev_move.to as usize;
        self.table[piece_index][to_index] = Some(cutoff_move);
    }
}
