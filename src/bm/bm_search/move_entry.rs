use cozy_chess::Move;

#[derive(Debug, Copy, Clone)]
struct Killer {
    mv: Move,
    score: i16,
}

impl Killer {
    fn mv(self) -> Move {
        self.mv
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MoveEntry {
    moves: [Option<Killer>; 2],
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
            .position(|&maybe_mv| maybe_mv.map(Killer::mv) == Some(mv))
    }

    pub fn get(&self, index: usize) -> Option<Move> {
        self.moves[index].map(Killer::mv)
    }

    pub fn remove(&mut self, index: usize) -> Option<Move> {
        self.moves[index].take().map(Killer::mv)
    }

    pub fn push(&mut self, mv: Move, score: i16) {
        let replace_score = score * 2;
        let mut index = 1;
        if self.moves[0].map_or(true, |mv_0| {
            self.moves[1].map_or(false, |mv_1| mv_0.score < mv_1.score)
        }) {
            index = 0;
        }

        if let Some(killer) = &mut self.moves[index] {
            if replace_score >= killer.score {
                killer.mv = mv;
                killer.score = score;
            }
        } else {
            self.moves[index] = Some(Killer { mv, score });
        }
    }

    pub fn contains(&self, mv: Move) -> bool {
        self.moves
            .iter()
            .any(|&maybe_mv| Some(mv) == maybe_mv.map(Killer::mv))
    }
}
