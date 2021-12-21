use chess::{Board, ChessMove, Color, Piece, Square};

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
        let to_index = to.to_index();
        self.table[piece_index][to_index]
    }

    pub fn cutoff(&mut self, board: &Board, make_move: ChessMove, fails: &[ChessMove], amt: u32) {
        if amt > 20 {
            return;
        }
        let piece = board.piece_on(make_move.get_source()).unwrap();
        let index = piece_index(board.side_to_move(), piece);
        let to_index = make_move.get_dest().to_index();

        let value = self.table[index][to_index];
        let change = (amt * amt) as i16;
        let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;

        let increment = change - decay;

        self.table[index][to_index] += increment;

        for &quiet in fails {
            let piece = board.piece_on(quiet.get_source()).unwrap();
            let index = piece_index(board.side_to_move(), piece);
            let to_index = quiet.get_dest().to_index();
            let value = self.table[index][to_index];
            let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;
            let decrement = change + decay;

            self.table[index][to_index] -= decrement;
        }
    }
}

#[derive(Debug, Clone)]
pub struct CounterMoveTable {
    table: Box<[[Option<ChessMove>; SQUARE_COUNT]; PIECE_COUNT]>,
}

impl CounterMoveTable {
    pub fn new() -> Self {
        Self {
            table: Box::new([[None; SQUARE_COUNT]; PIECE_COUNT]),
        }
    }

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> Option<ChessMove> {
        let piece_index = piece_index(color, piece);
        let to_index = to.to_index();
        self.table[piece_index][to_index]
    }

    pub fn cutoff(
        &mut self,
        board: &Board,
        prev_move: ChessMove,
        cutoff_move: ChessMove,
        amt: u32,
    ) {
        if amt > 20 {
            return;
        }
        let piece = board.piece_on(prev_move.get_dest()).unwrap();
        let piece_index = piece_index(board.side_to_move(), piece);
        let to_index = prev_move.get_dest().to_index();
        self.table[piece_index][to_index] = Some(cutoff_move);
    }
}

#[derive(Debug, Clone)]
pub struct DoubleMoveHistory {
    table: Box<[[[[i16; SQUARE_COUNT]; PIECE_COUNT]; SQUARE_COUNT]; PIECE_COUNT]>,
}

impl DoubleMoveHistory {
    pub fn new() -> Self {
        Self {
            table: Box::new([[[[0; SQUARE_COUNT]; PIECE_COUNT]; SQUARE_COUNT]; PIECE_COUNT]),
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
        let to_0_index = to_0.to_index();
        let piece_1_index = piece_index(color, piece_1);
        let to_1_index = to_1.to_index();
        self.table[piece_0_index][to_0_index][piece_1_index][to_1_index]
    }

    pub fn cutoff(
        &mut self,
        board: &Board,
        prev_move: ChessMove,
        make_move: ChessMove,
        fails: &[ChessMove],
        amt: u32,
    ) {
        if amt > 20 {
            return;
        }
        let prev_piece = board.piece_on(prev_move.get_dest()).unwrap();
        let prev_index = piece_index(board.side_to_move(), prev_piece);
        let prev_to_index = prev_move.get_dest().to_index();

        let piece = board.piece_on(make_move.get_source()).unwrap();
        let index = piece_index(board.side_to_move(), piece);
        let to_index = make_move.get_dest().to_index();

        let value = self.table[prev_index][prev_to_index][index][to_index];
        let change = (amt * amt) as i16;
        let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;

        let increment = change - decay;

        self.table[prev_index][prev_to_index][index][to_index] += increment;

        for &quiet in fails {
            let piece = board.piece_on(quiet.get_source()).unwrap();
            let index = piece_index(board.side_to_move(), piece);
            let to_index = quiet.get_dest().to_index();
            let value = self.table[prev_index][prev_to_index][index][to_index];
            let decay = (change as i32 * value as i32 / MAX_VALUE) as i16;
            let decrement = change + decay;

            self.table[prev_index][prev_to_index][index][to_index] -= decrement;
        }
    }
}

fn piece_index(color: Color, piece: Piece) -> usize {
    color.to_index() * PIECE_COUNT / 2 + piece.to_index()
}