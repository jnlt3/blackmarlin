use cozy_chess::{Board, Color, Move, Piece, Square};

pub const MAX_VALUE: i32 = 512;
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

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> i16 {
        let piece_index = piece_index(color, piece);
        let to_index = to as usize;
        self.table[piece_index][to_index]
    }

    pub fn cutoff(&mut self, board: &Board, make_move: Move, fails: &[Move], amt: u32) {
        let amt = amt.min(20);
        let piece = board.piece_on(make_move.from).unwrap();
        let index = piece_index(board.side_to_move(), piece);
        let to_index = make_move.to as usize;

        let value = self.table[index][to_index];
        let change = (amt * amt) as i16;
        let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;

        let increment = change - decay;

        self.table[index][to_index] += increment;

        for &quiet in fails {
            let piece = board.piece_on(quiet.from).unwrap();
            let index = piece_index(board.side_to_move(), piece);
            let to_index = quiet.to as usize;
            let value = self.table[index][to_index];
            let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;
            let decrement = change + decay;

            self.table[index][to_index] -= decrement;
        }
    }
}

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
        let piece_index = piece_index(color, piece);
        let to_index = to as usize;
        self.table[piece_index][to_index]
    }

    pub fn cutoff(&mut self, board: &Board, prev_move: Move, cutoff_move: Move, amt: u32) {        let piece = board.piece_on(prev_move.to).unwrap_or(Piece::King);
        let piece_index = piece_index(board.side_to_move(), piece);
        let to_index = prev_move.to as usize;
        self.table[piece_index][to_index] = Some(cutoff_move);
    }
}

#[derive(Debug, Clone)]
pub struct DoubleMoveHistory {
    table: Box<[[[[i16; SQUARE_COUNT]; PIECE_COUNT / 2]; SQUARE_COUNT]; PIECE_COUNT]>,
}

impl DoubleMoveHistory {
    pub fn new() -> Self {
        Self {
            table: Box::new([[[[0; SQUARE_COUNT]; PIECE_COUNT / 2]; SQUARE_COUNT]; PIECE_COUNT]),
        }
    }

    pub fn get(
        &self,
        color: Color,
        piece_0: Piece,
        to_0: Square,
        piece_1: Piece,
        to_1: Square,
    ) -> i16 {
        let piece_0_index = piece_index(color, piece_0);
        let to_0_index = to_0 as usize;
        let piece_1_index = piece_1 as usize;
        let to_1_index = to_1 as usize;
        self.table[piece_0_index][to_0_index][piece_1_index][to_1_index]
    }

    pub fn cutoff(
        &mut self,
        board: &Board,
        prev_move: Move,
        make_move: Move,
        fails: &[Move],
        amt: u32,
    ) {
        let amt = amt.min(20);
        let prev_piece = board.piece_on(prev_move.to).unwrap_or(Piece::King);
        let prev_index = piece_index(board.side_to_move(), prev_piece);
        let prev_to_index = prev_move.to as usize;

        let piece = board.piece_on(make_move.from).unwrap();
        let index = piece as usize;
        let to_index = make_move.to as usize;

        let value = self.table[prev_index][prev_to_index][index][to_index];
        let change = (amt * amt) as i16;
        let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;

        let increment = change - decay;

        self.table[prev_index][prev_to_index][index][to_index] += increment;

        for &quiet in fails {
            let piece = board.piece_on(quiet.from).unwrap();
            let index = piece as usize;
            let to_index = quiet.to as usize;
            let value = self.table[prev_index][prev_to_index][index][to_index];
            let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;
            let decrement = change + decay;

            self.table[prev_index][prev_to_index][index][to_index] -= decrement;
        }
    }
}

fn piece_index(color: Color, piece: Piece) -> usize {
    color as usize * PIECE_COUNT / 2 + piece as usize
}
