use cozy_chess::Move;

#[derive(Debug, Copy, Clone)]
pub struct MoveEntry {
    moves: [Option<Move>; 2],
}

impl MoveEntry {
    pub fn new() -> Self {
        Self { moves: [None; 2] }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn index_of(&self, mv: Move) -> Option<usize> {
        self.moves.iter().position(|&maybe_mv| maybe_mv == Some(mv))
    }

    pub fn get(&self, index: usize) -> Option<Move> {
        self.moves[index]
    }

    pub fn remove(&mut self, index: usize) -> Option<Move> {
        self.moves[index].take()
    }

    pub fn push(&mut self, mv: Move) {
        self.moves[1] = self.moves[0];
        self.moves[0] = Some(mv);
    }

    pub fn contains(&self, mv: Move) -> bool {
        self.moves.iter().any(|&maybe_mv| Some(mv) == maybe_mv)
    }
}
