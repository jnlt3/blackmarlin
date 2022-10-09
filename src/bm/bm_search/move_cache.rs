use cozy_chess::Board;

use crate::bm::bm_runner::ab_runner::MoveData;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// from: 6, to: 6, capture: 1, piece: 3
struct Entry {
    data: u16,
}

impl Entry {
    fn new_invalid() -> Self {
        Entry { data: 0 }
    }

    fn from_move_data(move_data: MoveData) -> Self {
        let mut data = 0;
        data |= move_data.from as u16;
        data |= (move_data.to as u16) << 6;
        data |= (move_data.capture as u16) << 12;
        data |= (move_data.piece as u16) << 13;
        Entry { data }
    }
}

#[derive(Debug, Clone)]
pub struct MoveCache {
    cache: Box<[Entry; 65536]>,
}

impl MoveCache {
    pub fn new() -> Self {
        Self {
            cache: Box::new([Entry::new_invalid(); 65536]),
        }
    }

    pub fn has(&self, board: &Board, mv: MoveData) -> bool {
        let index = (board.hash() as usize) >> 48;
        let move_data = Entry::from_move_data(mv);
        self.cache[index ^ (move_data.data as usize)] == move_data
    }

    pub fn add(&mut self, board: &Board, mv: MoveData) {
        let index = (board.hash() as usize) >> 48;
        let move_data = Entry::from_move_data(mv);
        self.cache[index ^ (move_data.data as usize)] = Entry::from_move_data(mv)
    }
}
