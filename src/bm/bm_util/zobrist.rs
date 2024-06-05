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
    stm: u16,
}

impl Zobrist {
    pub fn new() -> Self {
        let mut xor_shift = XorShift16::new();
        let mut hashes = [[0; Square::NUM]; Color::NUM];
        for color in &mut hashes {
            for square in color {
                *square = xor_shift.next();
            }
        }
        Self {
            stack: vec![],
            current: 0,
            hashes,
            stm: xor_shift.next(),
        }
    }

	pub fn hash(&self) -> u16 {
		self.current
	}

    pub fn null_move(&mut self) {
        self.current ^= self.stm;
        self.stack.push(self.current)
    }

    pub fn make_move(&mut self, w_diff: BitBoard, b_diff: BitBoard) {
        for w in w_diff {
            self.current ^= self.hashes[0][w as usize];
        }
        for b in b_diff {
            self.current ^= self.hashes[1][b as usize];
        }
        self.current ^= self.stm;
        self.stack.push(self.current);
    }

    pub fn unmake_move(&mut self) {
        self.current = self.stack.pop().unwrap();
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.current = 0;
    }
}
