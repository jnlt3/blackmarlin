use cozy_chess::{Color, Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Avg<const F: i32, const D: i32> {
    sum_eval: i16,
}

impl<const F: i32, const D: i32> std::ops::AddAssign<i16> for Avg<F, D> {
    fn add_assign(&mut self, rhs: i16) {
        self.sum_eval = ((self.sum_eval as i32 * (D - F) + rhs as i32 * F) / D) as i16
    }
}

#[derive(Debug, Clone)]
pub struct EvalTable {
    table: [[[Avg<1, 10>; 64]; 6]; 2],
}

impl EvalTable {
    pub fn new() -> Self {
        Self {
            table: [[[Default::default(); 64]; 6]; 2],
        }
    }

    pub fn add(&mut self, color: Color, piece: Piece, to: Square, eval_diff: i16) {
        self.table[color as usize][piece as usize][to as usize] += eval_diff;
    }

    pub fn get(&self, color: Color, piece: Piece, to: Square) -> i16 {
        self.table[color as usize][piece as usize][to as usize].sum_eval as i16
    }
}
