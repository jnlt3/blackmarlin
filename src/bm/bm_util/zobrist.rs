use cozy_chess::{BitBoard, Color, Square};

struct XorShift16 {
    state: u16,
}

impl XorShift16 {
    fn new() -> Self {
        Self { state: 1 }
    }

    fn next(&mut self) -> u16 {
        self.state ^= self.state << 7;
        self.state ^= self.state >> 9;
        self.state ^= self.state << 8;
        self.state
    }
}

#[derive(Debug, Clone)]
pub struct Zobrist {
    stack: Vec<u16>,
    current: u16,
    hashes: [[u16; Square::NUM]; Color::NUM],
}

impl Zobrist {
    pub fn new(w: BitBoard, b: BitBoard) -> Self {
        let mut xor_shift = XorShift16::new();
        let mut hashes = [[0; Square::NUM]; Color::NUM];
        for color in &mut hashes {
            for square in color {
                *square = xor_shift.next();
            }
        }
        let mut zobrist = Self {
            stack: vec![],
            current: 0,
            hashes,
        };
        zobrist.clear(w, b);
        zobrist
    }

    pub fn hash(&self) -> u16 {
        self.current
    }

    pub fn null_move(&mut self) {
        self.stack.push(self.current)
    }

    pub fn make_move(&mut self, w_diff: BitBoard, b_diff: BitBoard) {
        self.stack.push(self.current);
        for w in w_diff {
            self.current ^= self.hashes[0][w as usize];
        }
        for b in b_diff {
            self.current ^= self.hashes[1][b as usize];
        }
    }

    pub fn unmake_move(&mut self) {
        self.current = self.stack.pop().unwrap();
    }

    pub fn clear(&mut self, w: BitBoard, b: BitBoard) {
        self.stack.clear();
        self.current = 0;
        for w in w {
            self.current ^= self.hashes[0][w as usize];
        }
        for b in b {
            self.current ^= self.hashes[1][b as usize];
        }
    }
}
