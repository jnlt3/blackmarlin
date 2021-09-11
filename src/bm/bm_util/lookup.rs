use std::mem;

#[derive(Copy, Clone, Debug)]
pub struct LookUp<T: Copy, const DEPTH: usize> {
    table: [T; DEPTH],
}

impl<T: Copy, const DEPTH: usize> LookUp<T, DEPTH> {
    pub fn new<F: Fn(usize) -> T>(init: F) -> Self {
        let mut table: [T; DEPTH] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        for (depth, value) in table.iter_mut().enumerate() {
            *value = init(depth);
        }
        Self { table }
    }

    pub fn get(&self, depth: usize) -> T {
        self.table[depth.min(DEPTH - 1)]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LookUp2d<T: Copy, const DEPTH: usize, const MOVE: usize> {
    table: [[T; MOVE]; DEPTH],
}

impl<T: Copy, const DEPTH: usize, const MOVE: usize> LookUp2d<T, DEPTH, MOVE> {
    pub fn new<F: Fn(usize, usize) -> T>(init: F) -> Self {
        let mut table: [[T; MOVE]; DEPTH] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        for (depth, moves) in table.iter_mut().enumerate() {
            for (mv, value) in moves.iter_mut().enumerate() {
                *value = init(depth, mv);
            }
        }
        Self { table }
    }

    pub fn get(&self, depth: usize, mv: usize) -> T {
        self.table[depth.min(DEPTH - 1)][mv.min(MOVE - 1)]
    }
}
