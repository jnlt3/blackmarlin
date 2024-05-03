use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

use cozy_chess::{Board, Move, Piece, Square};

use crate::bm::bm_util::eval::Evaluation;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TTMove(u16);

impl TTMove {
    fn new(make_move: Move) -> Self {
        let mut bits = 0;
        bits |= make_move.from as u16;
        bits |= (make_move.to as u16) << 6;
        bits |= (make_move.promotion.map_or(0b1111, |piece| piece as u16)) << 12;
        Self(bits)
    }

    fn to_move(self) -> Move {
        const MASK_4: u16 = 0b1111;
        const MASK_6: u16 = 0b111111;
        let bits = self.0;

        let promotion = (bits >> 12) & MASK_4;
        let promotion = if promotion == 0b1111 {
            None
        } else {
            Piece::try_index(promotion as usize)
        };

        Move {
            from: Square::index((bits & MASK_6) as usize),
            to: Square::index(((bits >> 6) & MASK_6) as usize),
            promotion,
        }
    }
}

#[test]
fn compressed_moves() {
    let board = Board::default();
    board.generate_moves(|piece_moves| {
        for make_move in piece_moves {
            assert_eq!(make_move, TTMove::new(make_move).to_move());
        }
        false
    });
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntryType {
    Missing,
    Entry { bounds: Bounds },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bounds {
    LowerBound,
    Exact,
    UpperBound,
}

impl EntryType {
    fn to_u16(self) -> u16 {
        match self {
            EntryType::Missing => 0,
            EntryType::Entry { bounds } => match bounds {
                Bounds::LowerBound => 1,
                Bounds::Exact => 2,
                Bounds::UpperBound => 3,
            },
        }
    }

    fn from_u16(val: u16) -> EntryType {
        match val {
            1..=3 => {
                let val = val - 1;
                let bounds = match val {
                    0 => Bounds::LowerBound,
                    1 => Bounds::Exact,
                    2 => Bounds::UpperBound,
                    _ => unreachable!(),
                };
                EntryType::Entry { bounds }
            }
            _ => EntryType::Missing,
        }
    }
}

type Layout = (u8, u16, i16, u16, u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub depth: u32,
    pub bounds: Bounds,
    pub score: Evaluation,
    pub table_move: Move,
    pub age: u8,
}

impl Analysis {
    fn from_raw(bits: u64) -> Option<Self> {
        let (depth, entry_type, score, table_move, age) =
            unsafe { std::mem::transmute::<_, Layout>(bits) };
        let entry = EntryType::from_u16(entry_type);

        match entry {
            EntryType::Missing => None,
            EntryType::Entry { bounds } => Some(Self {
                depth: depth as u32,
                bounds,
                score: Evaluation::new(score),
                table_move: TTMove(table_move).to_move(),
                age,
            }),
        }
    }

    fn to_raw(self) -> u64 {
        unsafe {
            let entry = EntryType::Entry {
                bounds: self.bounds,
            };
            std::mem::transmute::<Layout, u64>((
                self.depth as u8,
                entry.to_u16(),
                self.score.raw(),
                TTMove::new(self.table_move).0,
                self.age,
            ))
        }
    }

    fn new(depth: u32, bounds: Bounds, score: Evaluation, table_move: Move, age: u8) -> Self {
        Self {
            depth,
            bounds,
            score,
            table_move,
            age,
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    hash: AtomicU32,
    analysis: [AtomicU32; 2],
}

impl Entry {
    fn zeroed() -> Self {
        Self {
            hash: AtomicU32::new(0),
            analysis: [AtomicU32::new(0), AtomicU32::new(0)],
        }
    }
    fn zero(&self) {
        self.hash.store(0, Ordering::Relaxed);
        self.analysis[0].store(0, Ordering::Relaxed);
        self.analysis[1].store(0, Ordering::Relaxed);
    }

    fn set_new(&self, hash: u32, entry: u64) {
        self.hash.store(hash, Ordering::Relaxed);
        self.analysis[0].store(entry as u32, Ordering::Relaxed);
        self.analysis[1].store((entry >> 32) as u32, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct TranspositionTable {
    table: Box<[Entry]>,
    age: AtomicU8,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        let table = (0..size).map(|_| Entry::zeroed()).collect::<Box<_>>();
        Self {
            table,
            age: AtomicU8::new(0),
        }
    }

    // From Viridithas - Cosmo
    fn tt_index(&self, hash: u64) -> usize {
        let key = u128::from(hash);
        let len = self.table.len() as u128;
        // fixed-point multiplication trick!
        ((key * len) >> 64) as usize
    }

    fn tt_hash(&self, hash: u64) -> u32 {
        hash as u32
    }

    #[cfg(not(target_feature = "sse"))]
    pub fn prefetch(&self, _: &Board) {}

    #[cfg(target_feature = "sse")]
    pub fn prefetch(&self, board: &Board) {
        use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        let hash = board.hash();
        let index = self.tt_index(hash);
        unsafe {
            let ptr = self.table.as_ptr().add(index);
            _mm_prefetch(ptr as *const i8, _MM_HINT_T0);
        }
    }

    pub fn get(&self, board: &Board) -> Option<Analysis> {
        let hash = board.hash();
        let tt_hash = self.tt_hash(hash);
        let tt_index = self.tt_index(hash);

        let entry = &self.table[tt_index];
        let entry_hash = entry.hash.load(Ordering::Relaxed);
        let entry_a = entry.analysis[0].load(Ordering::Relaxed);
        let entry_b = entry.analysis[1].load(Ordering::Relaxed);
        let entry_u64 = entry_a as u64 | ((entry_b as u64) << 32);
        if tt_hash == entry_hash {
            return Analysis::from_raw(entry_u64);
        }
        None
    }

    pub fn set(
        &self,
        board: &Board,
        depth: u32,
        entry_type: Bounds,
        score: Evaluation,
        table_move: Move,
    ) {
        let new = Analysis::new(
            depth,
            entry_type,
            score,
            table_move,
            self.age.load(Ordering::Relaxed),
        );
        let hash = board.hash();
        let tt_hash = self.tt_hash(hash);
        let tt_index = self.tt_index(hash);
        let fetched_entry = &self.table[tt_index];
        let entry_a = fetched_entry.analysis[0].load(Ordering::Relaxed);
        let entry_b = fetched_entry.analysis[1].load(Ordering::Relaxed);
        let entry_u64 = entry_a as u64 | ((entry_b as u64) << 32);
        let previous = Analysis::from_raw(entry_u64);
        if previous.map_or(true, |previous| self.replace(&new, &previous)) {
            let analysis_u64 = new.to_raw();
            fetched_entry.set_new(tt_hash, analysis_u64);
        }
    }

    fn age_of(&self, analysis: &Analysis) -> u8 {
        let age = self.age.load(Ordering::Relaxed);
        age.wrapping_sub(analysis.age)
    }

    fn replace(&self, new: &Analysis, prev: &Analysis) -> bool {
        fn extra_depth(analysis: &Analysis) -> u32 {
            // +1 depth for Exact scores and lower bounds
            matches!(analysis.bounds, Bounds::Exact | Bounds::LowerBound) as u32
        }

        let new_depth = new.depth + extra_depth(new);
        let prev_depth = prev.depth + extra_depth(prev);

        new_depth * 2 + self.age_of(prev) as u32 + 1 >= prev_depth
    }

    pub fn clean(&self) {
        self.age.store(0, Ordering::Relaxed);
        self.table.iter().for_each(|entry| entry.zero());
    }

    pub fn age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }
}
