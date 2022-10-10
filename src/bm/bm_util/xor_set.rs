use cozy_chess::Board;

use crate::bm::bm_runner::ab_runner::MoveData;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// from: 6, to: 6, capture: 1, piece: 3
struct Entry {
    data: u16,
}

fn from_move_data(move_data: MoveData) -> u16 {
    let mut data = 0;
    data |= move_data.from as u16;
    data |= (move_data.to as u16) << 6;
    data |= (move_data.capture as u16) << 12;
    data |= (move_data.piece as u16) << 13;
    data
}

#[derive(Debug, Clone)]
pub struct XorSet {
    cache: Box<[u16]>,
}

impl XorSet {
    pub fn new() -> Self {
        Self {
            cache: vec![0; 65536].into_boxed_slice(),
        }
    }

    pub fn insert(&mut self, board: &Board, mv: MoveData) {
        let hash = (board.hash() >> 48) as u16;
        let value = from_move_data(mv);
        let index = (hash ^ value) as usize;
        self.cache[index] = value
    }

    pub fn has(&self, board: &Board, mv: MoveData) -> bool {
        let hash = (board.hash() >> 48) as u16;
        let value = from_move_data(mv);
        let index = (hash ^ value) as usize;
        self.cache[index] == value
    }
}
