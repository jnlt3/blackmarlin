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

#[derive(Copy, Clone)]
pub struct HistoryFetched {
    quiet: *mut Butterfly<i16>,
    capture: *mut Butterfly<i16>,
    counter_move: Option<*mut PieceTo<i16>>,
}

impl HistoryFetched {
    pub fn get_quiet(&self, make_move: Move) -> i16 {
        unsafe { (*self.quiet)[make_move.from as usize][make_move.to as usize] }
    }

    pub fn get_capture(&self, make_move: Move) -> i16 {
        unsafe { (*self.capture)[make_move.from as usize][make_move.to as usize] }
    }

    pub fn get_counter_move(&self, pos: &Position, make_move: Move) -> i16 {
        if let Some(counter_move) = self.counter_move {
            unsafe {
                (*counter_move)[pos.board().piece_on(make_move.from).unwrap() as usize]
                    [make_move.to as usize]
            }
        } else {
            0
        }
    }

    pub fn update_quiet(&mut self, pos: &Position, make_move: Move, fails: &[Move], amt: i16) {
        let from = make_move.from as usize;
        let to = make_move.to as usize;

        unsafe {
            bonus(&mut (*self.quiet)[from][to], amt);
            for make_move in fails {
                let from = make_move.from as usize;
                let to = make_move.to as usize;
                malus(&mut (*self.quiet)[from][to], amt);
            }
            if let Some(counter_move) = self.counter_move {
                let piece = pos.board().piece_on(make_move.from).unwrap() as usize;
                bonus(&mut (*counter_move)[piece][to], amt);
                for make_move in fails {
                    let piece = pos.board().piece_on(make_move.from).unwrap() as usize;
                    let to = make_move.to as usize;
                    malus(&mut (*counter_move)[piece][to], amt);
                }
            }
        }
    }

    pub fn update_captures(&mut self, make_move: Move, fails: &[Move], amt: i16) {
        let from = make_move.from as usize;
        let to = make_move.to as usize;

        unsafe {
            bonus(&mut (*self.capture)[from][to], amt);
            for make_move in fails {
                let from = make_move.from as usize;
                let to = make_move.to as usize;
                malus(&mut (*self.capture)[from][to], amt);
            }
        }
    }
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

    #[inline(never)]
    pub fn fetch_hist(&mut self, pos: &Position, prev_move: Option<Move>) -> HistoryFetched {
        let stm = pos.board().side_to_move() as usize;
        let quiet = &mut self.quiet[stm] as *mut _;
        let capture = &mut self.capture[stm] as *mut _;
        let counter_move = if let Some(prev_move) = prev_move {
            let piece = pos.board().piece_on(prev_move.to).unwrap_or(Piece::King);
            Some(&mut self.counter_move[stm][piece as usize][prev_move.to as usize] as *mut _)
        } else {
            None
        };
        HistoryFetched {
            quiet,
            capture,
            counter_move,
        }
    }
}
