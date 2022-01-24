use std::sync::atomic::{AtomicU64, Ordering};

use cozy_chess::{Board, Move, Square};

use crate::bm::bm_eval::eval::Evaluation;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntryType {
    LowerBound,
    Exact,
    UpperBound,
}

#[derive(Debug, Copy, Clone)]
pub struct Analysis {
    depth: u8,
    entry_type: Option<EntryType>,
    score: Evaluation,
    table_move: Move,
}

impl Analysis {
    pub fn new(depth: u32, entry_type: EntryType, score: Evaluation, table_move: Move) -> Self {
        Self {
            depth: depth as u8,
            entry_type: Some(entry_type),
            score,
            table_move,
        }
    }

    #[inline]
    pub fn depth(&self) -> u32 {
        self.depth as u32
    }

    #[inline]
    pub fn entry_type(&self) -> EntryType {
        self.entry_type.unwrap()
    }

    #[inline]
    pub fn score(&self) -> Evaluation {
        self.score
    }

    #[inline]
    pub fn table_move(&self) -> Move {
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
            hash: AtomicU64::new(u64::MAX),
            analysis: std::mem::transmute::<Analysis, AtomicU64>(Analysis {
                depth: 0,
                entry_type: None,
                score: Evaluation::new(0),
                table_move: Move {
                    from: Square::A1,
                    to: Square::A1,
                    promotion: None,
                },
            }),
        }
    }
    fn zero(&self) {
        self.hash.store(u64::MAX, Ordering::Relaxed);
        self.analysis.store(0, Ordering::Relaxed);
    }

    fn set_new(&self, hash: u64, entry: u64) {
        self.hash.store(hash, Ordering::Relaxed);
        self.analysis.store(entry, Ordering::Relaxed);
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

    pub fn get(&self, board: &Board) -> Option<Analysis> {
        let hash = board.hash();
        let index = self.index(hash);

        let entry = &self.table[index];
        let hash_u64 = entry.hash.load(Ordering::Relaxed);
        let entry_u64 = entry.analysis.load(Ordering::Relaxed);
        if entry_u64 ^ hash == hash_u64 {
            let analysis = unsafe { std::mem::transmute::<u64, Analysis>(entry_u64) };
            if analysis.entry_type.is_some() {
                Some(analysis)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set(&self, board: &Board, entry: Analysis) {
        let hash = board.hash();
        let index = self.index(hash);
        let fetched_entry = &self.table[index];
        let analysis: Analysis =
            unsafe { std::mem::transmute(fetched_entry.analysis.load(Ordering::Relaxed)) };
        if analysis.entry_type.is_none() || entry.depth >= analysis.depth / 2 {
            let analysis_u64 = unsafe { std::mem::transmute::<Analysis, u64>(entry) };
            fetched_entry.set_new(hash ^ analysis_u64, analysis_u64);
        }
    }

    pub fn clean(&self) {
        self.table.iter().for_each(|entry| entry.zero());
    }
}
