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

    fn lowest(&self) -> usize {
        let Some(killer_a) = self.moves[0] else {
            return 0;
        };
        let Some(killer_b) = self.moves[1] else {
            return 1;
        };
        match killer_a.score < killer_b.score {
            true => 0,
            false => 1,
        }
    }

    fn clear_dup(&mut self) {
        let Some(killer) = self.moves[0] else {
            return;
        };
        if Some(killer.mv) == self.moves[1].map(Killer::mv) {
            self.moves[1] = None;
        }
    }

    pub fn push(&mut self, mv: Move, score: i16) {
        let lowest = self.index_of(mv).unwrap_or(self.lowest());
        let other = 1 - lowest;
        if self.moves[lowest].is_none() {
            self.moves[lowest] = Some(Killer { mv, score });
            return;
        }
        self.moves[other] = self.moves[lowest];
        let killer = self.moves[lowest].as_mut().unwrap();
        if score * 2 > killer.score {
            killer.mv = mv;
            killer.score = score;
        }
        self.clear_dup();
    }

    pub fn contains(&self, mv: Move) -> bool {
        self.moves
            .iter()
            .any(|&maybe_mv| Some(mv) == maybe_mv.map(Killer::mv))
    }
}
