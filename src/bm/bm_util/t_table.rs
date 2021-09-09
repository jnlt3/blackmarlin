use std::sync::{Mutex};

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

const ENTRIES_PER_LOCK: usize = 64;

type EntryGroup = [Option<(u64, Analysis)>; ENTRIES_PER_LOCK];

#[derive(Debug)]
pub struct TranspositionTable {
    table: Vec<Mutex<EntryGroup>>,
    mask: usize,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two();
        assert_eq!(size % ENTRIES_PER_LOCK, 0);
        let table = (0..size / ENTRIES_PER_LOCK)
            .into_iter()
            .map(|_| Mutex::new([None; ENTRIES_PER_LOCK]))
            .collect::<Vec<_>>();
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
        let locked_table = &self.table[index / ENTRIES_PER_LOCK].lock().unwrap();
        if let Some((entry_hash, entry)) = &locked_table[index % ENTRIES_PER_LOCK] {
            if *entry_hash == hash {
                Some(*entry)
            } else {
                None
            }
        } else {
            None
        }
        /*
        let table = &*self.table.lock().unwrap();
        if let Some((entry_hash, entry)) = &self.table[index / ENTRIES_PER_LOCK] {
            if *entry_hash == hash {
                Some(entry.clone())
            } else {
                None
            }
        } else {
            None
        }

         */
    }

    pub fn set(&self, position: &Position, entry: &Analysis) {
        let hash = position.hash();
        let index = self.index(hash);
        let locked_table = &mut self.table[index / ENTRIES_PER_LOCK].lock().unwrap();
        let index = index % ENTRIES_PER_LOCK;
        /*
        if let Some((entry_hash, prev_entry)) = &mut locked_table[index] {
            if *entry_hash == hash {
                if entry.depth >= prev_entry.depth {
                    *prev_entry = entry.clone();
                }
                return;
            }
        }

         */
        locked_table[index] = Some((hash, *entry));
        /*
        let table = &mut *self.table.lock().unwrap();
        if let Some((entry_hash, prev_entry)) = &mut table[index] {
            if *entry_hash == hash {
                if entry.depth >= prev_entry.depth {
                    *prev_entry = entry.clone();
                }
                return;
            }
        }
        table[index] = Some((hash, entry.clone()));
         */
    }

    pub fn clean(&self) {
        self.table
            .iter()
            .for_each(|entry| *entry.lock().unwrap() = [None; ENTRIES_PER_LOCK]);
    }
}
