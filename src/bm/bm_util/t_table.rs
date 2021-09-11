use std::sync::Mutex;

use chess::ChessMove;

use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_util::position::Position;

#[derive(Debug, Copy, Clone)]
pub enum Score {
    LowerBound(Evaluation),
    Exact(Evaluation),
    UpperBound(Evaluation),
}

impl Score {
    pub fn value(&self) -> Evaluation {
        match self {
            Score::LowerBound(score) | Score::Exact(score) | Score::UpperBound(score) => *score,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Analysis {
    depth: u8,
    score: Score,
    table_move: ChessMove,
}

impl Analysis {
    pub fn new(depth: u32, score: Score, table_move: ChessMove) -> Self {
        Self {
            depth: depth as u8,
            score,
            table_move,
        }
    }

    #[inline]
    pub fn depth(&self) -> u32 {
        self.depth as u32
    }

    #[inline]
    pub fn score(&self) -> Score {
        self.score
    }

    #[inline]
    pub fn table_move(&self) -> ChessMove {
        self.table_move
    }
}

type Entry = Option<(u64, Analysis)>;

#[derive(Debug)]
pub struct TranspositionTable {
    table: Mutex<Vec<Entry>>,
    mask: usize,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two();
        let table = Mutex::new(vec![None; size]);
        Self {
            table,
            mask: size - 1,
        }
    }

    #[inline]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & self.mask
    }

    pub fn get(&self, position: &Position) -> Option<Analysis> {
        let hash = position.hash();
        let index = self.index(hash);
        if let Some((entry_hash, entry)) = &self.table.lock().unwrap()[index] {
            if *entry_hash == hash {
                Some(*entry)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set(&self, position: &Position, entry: &Analysis) {
        let hash = position.hash();
        let index = self.index(hash);
        let fetched_entry = &mut self.table.lock().unwrap()[index];
        *fetched_entry = Some((hash, *entry))
    }

    pub fn clean(&self) {
        self.table
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|entry| *entry = None);
    }
}
