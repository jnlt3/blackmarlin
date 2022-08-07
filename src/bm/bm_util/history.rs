use super::position::Position;
use cozy_chess::{Move, Piece};

pub const MAX_HIST: i16 = 512;

const SIDE_TO_MOVE: usize = 2;
const SQUARE: usize = 64;
const PIECE: usize = 6;

type Butterfly<T> = [[T; SQUARE]; SQUARE];
type PieceTo<T> = [[T; SQUARE]; PIECE];

fn hist_stat(amt: i16) -> i16 {
    (amt * amt).min(MAX_HIST)
}

fn bonus(hist: &mut i16, amt: i16) {
    let change = hist_stat(amt);
    let decay = (change as i32 * (*hist) as i32 / MAX_HIST as i32) as i16;
    let increment = change - decay;
    *hist += increment;
}

fn malus(hist: &mut i16, amt: i16) {
    let change = hist_stat(amt);
    let decay = (change as i32 * (*hist) as i32 / MAX_HIST as i32) as i16;
    let decrement = change + decay;
    *hist -= decrement;
}

#[derive(Debug, Clone)]
pub struct History {
    quiet: Box<[Butterfly<i16>; SIDE_TO_MOVE]>,
    capture: Box<[Butterfly<i16>; SIDE_TO_MOVE]>,
    counter_move: Box<[PieceTo<PieceTo<i16>>; SIDE_TO_MOVE]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            quiet: Box::new([[[0; SQUARE]; SQUARE]; SIDE_TO_MOVE]),
            capture: Box::new([[[0; SQUARE]; SQUARE]; SIDE_TO_MOVE]),
            counter_move: Box::new([[[[[0; SQUARE]; PIECE]; SQUARE]; PIECE]; SIDE_TO_MOVE]),
        }
    }

    pub fn get_all_quiet(&self, pos: &Position, make_move: Move, prev_move: Option<Move>) -> i16 {
        let mut value = self.get_quiet(pos, make_move);
        if let Some(prev_move) = prev_move {
            value += self.get_counter_move_hist(pos, prev_move, make_move);
        }
        value / 2
    }

    pub fn get_quiet(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        let from = make_move.from as usize;
        let to = make_move.to as usize;
        self.quiet[stm as usize][from][to]
    }

    pub fn get_capture(&self, pos: &Position, make_move: Move) -> i16 {
        let stm = pos.board().side_to_move();
        let from = make_move.from as usize;
        let to = make_move.to as usize;
        self.capture[stm as usize][from][to]
    }

    pub fn get_counter_move_hist(&self, pos: &Position, prev_move: Move, make_move: Move) -> i16 {
        if pos.len() == 0 {
            return 0;
        }
        let stm = pos.board().side_to_move() as usize;
        let current_piece = pos.board().piece_on(make_move.from).unwrap() as usize;
        let current_to = make_move.to as usize;
        let prev_piece = pos.board().piece_on(prev_move.to).unwrap_or(Piece::King) as usize;
        let prev_to = prev_move.to as usize;
        self.counter_move[stm][prev_piece][prev_to][current_piece][current_to]
    }

    pub fn update_quiet(&mut self, pos: &Position, make_move: Move, fails: &[Move], amt: i16) {
        let stm = pos.board().side_to_move() as usize;

        let from = make_move.from as usize;
        let to = make_move.to as usize;

        bonus(&mut self.quiet[stm][from][to], amt);
        for make_move in fails {
            let from = make_move.from as usize;
            let to = make_move.to as usize;
            malus(&mut self.quiet[stm][from][to], amt);
        }
    }

    pub fn update_counter_move(
        &mut self,
        pos: &Position,
        make_move: Move,
        fails: &[Move],
        prev_move: Move,
        amt: i16,
    ) {
        let stm = pos.board().side_to_move() as usize;

        let piece = pos.board().piece_on(make_move.from).unwrap() as usize;
        let to = make_move.to as usize;

        let prev_piece = pos.board().piece_on(prev_move.to).unwrap_or(Piece::King) as usize;
        let prev_to = prev_move.to as usize;
        bonus(
            &mut self.counter_move[stm][prev_piece][prev_to][piece][to],
            amt,
        );
        for make_move in fails {
            let piece = pos.board().piece_on(make_move.from).unwrap() as usize;
            let to = make_move.to as usize;
            malus(
                &mut self.counter_move[stm][prev_piece][prev_to][piece][to],
                amt,
            );
        }
    }

    pub fn update_capture(&mut self, pos: &Position, make_move: Move, fails: &[Move], amt: i16) {
        let stm = pos.board().side_to_move() as usize;
        let from = make_move.from as usize;
        let to = make_move.to as usize;
        bonus(&mut self.capture[stm][from][to], amt);
        for make_move in fails {
            let from = make_move.from as usize;
            let to = make_move.to as usize;
            malus(&mut self.capture[stm][from][to], amt);
        }
    }
}
