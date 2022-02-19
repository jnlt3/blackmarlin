use std::array::IntoIter;
use std::iter::{IntoIterator, Take};

use cozy_chess::{Move, Square};

#[derive(Debug, Copy, Clone)]
pub struct MoveEntry<const N: usize> {
    moves: [Move; N],
    index: usize,
    size: usize,
}

impl<const N: usize> MoveEntry<N> {
    pub fn new() -> Self {
        Self {
            moves: [Move {
                from: Square::A1,
                to: Square::A1,
                promotion: None,
            }; N],
            index: 0,
            size: 0,
        }
    }

    pub fn clear(&mut self) {
        self.index = 0;
        self.size = 0;
    }

    pub fn push(&mut self, killer_move: Move) {
        if N == 0 {
            return;
        }
        if self.size == 0 || !self.moves.contains(&killer_move) {
            self.moves[self.index] = killer_move;
            self.size += 1;
            self.index = (self.index + 1) % N;
        }
    }
}

pub type MoveEntryIterator<const N: usize> = Take<IntoIter<Move, N>>;

impl<const N: usize> IntoIterator for MoveEntry<N> {
    type Item = Move;
    type IntoIter = MoveEntryIterator<N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.moves).take(self.size)
    }
}
