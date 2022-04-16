use cozy_chess::Move;

use super::position::Position;

pub const MAX_VALUE: i32 = 512;
const STM_CNT: usize = 2;
const CAP_CNT: usize = 2;
const PIECE_CNT: usize = 6;
const SQ_CNT: usize = 64;

#[derive(Debug, Clone)]
pub struct History {
    piece_to: Box<[[[[i16; SQ_CNT]; PIECE_CNT]; CAP_CNT]; STM_CNT]>,
    counter_move: Box<[[[[[i16; SQ_CNT]; PIECE_CNT]; SQ_CNT]; PIECE_CNT]; STM_CNT]>,

    refutation: Box<[[[Option<Move>; SQ_CNT]; PIECE_CNT]; STM_CNT]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            piece_to: Box::new([[[[0_i16; SQ_CNT]; PIECE_CNT]; CAP_CNT]; STM_CNT]),
            counter_move: Box::new([[[[[0_i16; SQ_CNT]; PIECE_CNT]; SQ_CNT]; PIECE_CNT]; STM_CNT]),

            refutation: Box::new([[[None; SQ_CNT]; PIECE_CNT]; STM_CNT]),
        }
    }

    pub fn get_hist(&self, pos: &Position, make_move: Move) -> i16 {
        let board = pos.board();
        let stm = board.side_to_move();
        let is_capture = board.colors(!stm).has(make_move.to);
        let piece = board.piece_on(make_move.from).unwrap();

        self.piece_to[stm as usize][is_capture as usize][piece as usize][make_move.to as usize]
    }

    pub fn get_counter_move_hist(
        &self,
        pos: &Position,
        make_move: Move,
        prev_move: Option<Move>,
    ) -> i16 {
        if prev_move.is_none() || pos.prev_board().is_none() {
            return 0;
        }
        let prev_move = prev_move.unwrap();
        let prev_board = pos.prev_board().unwrap();
        let board = pos.board();
        let stm = board.side_to_move();

        let piece = board.piece_on(make_move.from).unwrap();
        let prev_piece = prev_board.piece_on(prev_move.from).unwrap();

        self.counter_move[stm as usize][prev_piece as usize][prev_move.to as usize][piece as usize]
            [make_move.to as usize]
    }

    pub fn get_refutation(&self, pos: &Position, prev_move: Option<Move>) -> Option<Move> {
        if prev_move.is_none() || pos.prev_board().is_none() {
            return None;
        }
        let prev_move = prev_move.unwrap();
        let prev_board = pos.prev_board().unwrap();
        let board = pos.board();
        let stm = board.side_to_move();

        let prev_piece = prev_board.piece_on(prev_move.from).unwrap();
        self.refutation[stm as usize][prev_piece as usize][prev_move.to as usize]
    }

    pub fn update(
        &mut self,
        pos: &Position,
        cutoff_move: Move,
        quiet_fails: &[Move],
        cap_fails: &[Move],
        prev_move: Option<Move>,
        depth: u32,
    ) {
        if depth > 20 {
            return;
        }
        let board = pos.board();
        let stm = board.side_to_move();
        let is_capture = board.colors(!stm).has(cutoff_move.to);
        let piece = board.piece_on(cutoff_move.from).unwrap();

        let weight = (depth * depth) as i16;

        let piece_to = &mut self.piece_to[stm as usize][is_capture as usize][piece as usize]
            [cutoff_move.to as usize];
        update_hist::<true>(piece_to, weight);

        let prev_piece = pos
            .prev_board()
            .zip(prev_move)
            .map_or(None, |(board, prev_move)| board.piece_on(prev_move.from));

        if let Some((prev_piece, prev_move)) = prev_piece.zip(prev_move) {
            let counter_move = &mut self.counter_move[stm as usize][prev_piece as usize]
                [prev_move.to as usize][piece as usize][cutoff_move.to as usize];
            update_hist::<true>(counter_move, weight);

            self.refutation[stm as usize][prev_piece as usize][prev_move.to as usize] =
                Some(cutoff_move);
        }

        if is_capture {
            for &fail in cap_fails {
                let piece = board.piece_on(fail.from).unwrap();
                let piece_to = &mut self.piece_to[stm as usize][is_capture as usize]
                    [piece as usize][fail.to as usize];
                update_hist::<false>(piece_to, weight);
            }
        } else {
            for &fail in quiet_fails {
                let piece = board.piece_on(fail.from).unwrap();
                let piece_to = &mut self.piece_to[stm as usize][is_capture as usize]
                    [piece as usize][fail.to as usize];
                update_hist::<false>(piece_to, weight);
                if let Some((prev_piece, prev_move)) = prev_piece.zip(prev_move) {
                    let counter_move = &mut self.counter_move[stm as usize][prev_piece as usize]
                        [prev_move.to as usize][piece as usize][fail.to as usize];
                    update_hist::<false>(counter_move, weight)
                }
            }
        }
    }
}

fn update_hist<const CUTOFF: bool>(value: &mut i16, weight: i16) {
    let decay = (weight as i32 * *value as i32 / MAX_VALUE) as i16;
    if CUTOFF {
        *value += weight - decay;
    } else {
        *value -= weight + decay;
    }
}
