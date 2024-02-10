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

    /// Returns the index of the move in the killer list or [None](None) if it doesn't exist
    pub fn index_of(&self, mv: Move) -> Option<usize> {
        self.moves.iter().position(|&maybe_mv| maybe_mv == Some(mv))
    }

    /// - Not guaranteed to be legal
    /// - Index should always be 0 or 1
    pub fn get(&self, index: usize) -> Option<Move> {
        self.moves[index]
    }

    /// - Index should always be 0 or 1
    pub fn remove(&mut self, index: usize) -> Option<Move> {
        self.moves[index].take()
    }

    /// Removes the least recent killer and ensures no duplicates
    pub fn push(&mut self, mv: Move) {
        if Some(mv) == self.moves[0] {
            return;
        }
        self.moves[1] = self.moves[0];
        self.moves[0] = Some(mv);
    }

    /// Returns true if a given move is a killer
    pub fn contains(&self, mv: Move) -> bool {
        self.moves.contains(&Some(mv))
    }
}
