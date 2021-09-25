use std::sync::atomic::{AtomicU64, Ordering};
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

#[derive(Debug)]
pub struct Entry {
    hash: AtomicU64,
    analysis: AtomicU64,
}

impl Entry {
    unsafe fn zeroed() -> Self {
        Self {
            hash: std::mem::transmute::<u64, AtomicU64>(u64::MAX),
            analysis: std::mem::transmute::<u64, AtomicU64>(0),
        }
    }
    unsafe fn zero(&self) {
        self.hash.store(u64::MAX, Ordering::SeqCst);
        self.analysis.store(0, Ordering::SeqCst);
    }

    unsafe fn new(hash: u64, entry: u64) -> Self {
        Self {
            hash: std::mem::transmute::<u64, AtomicU64>(hash ^ entry),
            analysis: std::mem::transmute::<u64, AtomicU64>(entry),
        }
    }

    unsafe fn set_new(&self, hash: u64, entry: u64) {
        self.hash.store(hash ^ entry, Ordering::SeqCst);
        self.analysis.store(entry, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct TranspositionTable {
    table: Box<[Entry]>,
    mask: usize,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two();
        let table = (0..size)
            .map(|_| unsafe { Entry::zeroed() })
            .collect::<Box<_>>();
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

        let entry = &self.table[index];
        let hash_u64 = entry.hash.load(Ordering::SeqCst);
        let entry_u64 = entry.analysis.load(Ordering::SeqCst);
        if entry_u64 ^ hash == hash_u64 {
            unsafe { Some(std::mem::transmute::<u64, Analysis>(entry_u64)) }
        } else {
            None
        }
    }

    pub fn set(&self, position: &Position, entry: &Analysis) {
        let hash = position.hash();
        let index = self.index(hash);
        let fetched_entry = &self.table[index];
        unsafe {
            fetched_entry.set_new(hash, std::mem::transmute::<Analysis, u64>(*entry));
        }
    }

    pub fn clean(&self) {
        self.table.iter().for_each(|entry| unsafe { entry.zero() });
    }
}
