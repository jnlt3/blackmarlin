#[derive(Copy, Clone, Debug)]
pub struct LookUp2d<T: Copy + Default, const DEPTH: usize, const MOVE: usize> {
    table: [[T; MOVE]; DEPTH],
}

impl<T: Copy + Default, const DEPTH: usize, const MOVE: usize> LookUp2d<T, DEPTH, MOVE> {
    pub fn new<F: Fn(usize, usize) -> T>(init: F) -> Self {
        let mut table: [[T; MOVE]; DEPTH] = [[Default::default(); MOVE]; DEPTH];
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
