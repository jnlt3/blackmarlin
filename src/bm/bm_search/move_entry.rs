use cozy_chess::Move;

#[derive(Debug, Copy, Clone, PartialEq)]
struct RatedMove {
    mv: Move,
    cutoff_margin: i16,
}

#[derive(Debug, Copy, Clone)]
pub struct MoveEntry {
    moves: [Option<RatedMove>; 2],
}

impl MoveEntry {
    pub fn new() -> Self {
        Self { moves: [None; 2] }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn index_of(&self, mv: Move) -> Option<usize> {
        self.moves
            .iter()
            .position(|&maybe_mv| maybe_mv.map(|rated_move| rated_move.mv) == Some(mv))
    }

    pub fn get(&self, index: usize) -> Option<Move> {
        self.moves[index].map(|rated_move| rated_move.mv)
    }

    pub fn remove(&mut self, index: usize) -> Option<Move> {
        self.moves[index].take().map(|rated_move| rated_move.mv)
    }

    pub fn push(&mut self, mv: Move, cutoff_margin: i16) {
        if Some(mv) == self.moves[0].map(|rated_move| rated_move.mv) {
            return;
        }
        if self.moves[1].map_or(false, |rated_move| {
            rated_move.cutoff_margin > cutoff_margin * 2
        }) {
            return;
        }

        let rated_move = RatedMove { mv, cutoff_margin };
        self.moves[1] = self.moves[0];
        self.moves[0] = Some(rated_move);
    }

    pub fn contains(&self, mv: Move) -> bool {
        self.moves
            .iter()
            .any(|&maybe_mv| Some(mv) == maybe_mv.map(|rated_move| rated_move.mv))
    }
}
