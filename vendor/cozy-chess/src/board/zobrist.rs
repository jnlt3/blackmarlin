use crate::*;

#[derive(Debug)]
struct ColorZobristConstants {
    pieces: [[u64; Square::NUM]; Piece::NUM],
    castle_rights: [u64; File::NUM]
}

#[derive(Debug)]
struct ZobristConstants {
    color: [ColorZobristConstants; Color::NUM],
    en_passant: [u64; File::NUM],
    black_to_move: u64
}

const ZOBRIST: ZobristConstants = {
    // Simple Pcg64Mcg impl
    let mut state = 0x7369787465656E2062797465206E756Du128 | 1;
    macro_rules! rand {
        () => {{
            state = state.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
            let rot = (state >> 122) as u32;
            let xsl = (state >> 64) as u64 ^ state as u64;
            xsl.rotate_right(rot)
        }};
    }

    macro_rules! fill_array {
        ($array:ident: $expr:expr) => {{
            let mut i = 0;
            while i < $array.len() {
                $array[i] = $expr;
                i += 1;
            }
        }};
    }

    macro_rules! color_zobrist_constant {
        () => {{
            let mut castle_rights = [0; File::NUM];
            fill_array!(castle_rights: rand!());

            let mut pieces = [[0; Square::NUM]; Piece::NUM];
            fill_array!(pieces: {
                let mut squares = [0; Square::NUM];
                fill_array!(squares: rand!());
                squares
            });
            
            ColorZobristConstants {
                pieces,
                castle_rights
            }
        }};
    }

    let mut en_passant = [0; File::NUM];
    fill_array!(en_passant: rand!());

    let white = color_zobrist_constant!();
    let black = color_zobrist_constant!();

    let black_to_move = rand!();

    ZobristConstants {
        color: [white, black],
        en_passant,
        black_to_move
    }
};

// This is Copy for performance reasons, since Copy guarantees a bit-for-bit copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZobristBoard {
    pieces: [BitBoard; Piece::NUM],
    colors: [BitBoard; Color::NUM],
    side_to_move: Color,
    castle_rights: [CastleRights; Color::NUM],
    en_passant: Option<File>,
    hash: u64
}

impl ZobristBoard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            pieces: [BitBoard::EMPTY; Piece::NUM],
            colors: [BitBoard::EMPTY; Color::NUM],
            side_to_move: Color::White,
            castle_rights: [CastleRights {
                short: None,
                long: None
            }; 2],
            en_passant: None,
            hash: 0
        }
    }

    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.pieces[piece as usize]
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.colors[color as usize]
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    pub fn castle_rights(&self, color: Color) -> &CastleRights {
        &self.castle_rights[color as usize]
    }

    #[inline(always)]
    pub fn en_passant(&self) -> Option<File> {
        self.en_passant
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    pub fn hash_without_ep(&self) -> u64 {
        let mut hash = self.hash;
        if let Some(file) = self.en_passant {
            hash ^= ZOBRIST.en_passant[file as usize];
        }
        hash
    }

    pub fn board_is_equal(&self, other: &Self) -> bool {
        self.pieces == other.pieces
            && self.colors == other.colors
            && self.side_to_move == other.side_to_move
            && self.castle_rights == other.castle_rights
    }

    #[inline(always)]
    pub fn xor_square(&mut self, piece: Piece, color: Color, square: Square) {
        let square_bb = square.bitboard();
        self.pieces[piece as usize] ^= square_bb;
        self.colors[color as usize] ^= square_bb;
        self.hash ^= ZOBRIST
            .color[color as usize]
            .pieces[piece as usize]
            [square as usize];
    }

    pub fn set_castle_right(&mut self, color: Color, short: bool, file: Option<File>)  {
        let rights = &mut self.castle_rights[color as usize];
        let right = if short {
            &mut rights.short
        } else {
            &mut rights.long
        };
        if let Some(prev) = core::mem::replace(right, file) {
            self.hash ^= ZOBRIST.color[color as usize].castle_rights[prev as usize];
        }
        if let Some(file) = file {
            self.hash ^= ZOBRIST.color[color as usize].castle_rights[file as usize];
        }
    }

    pub fn set_en_passant(&mut self, new_en_passant: Option<File>) {
        if let Some(file) = core::mem::replace(&mut self.en_passant, new_en_passant) {
            self.hash ^= ZOBRIST.en_passant[file as usize];
        }
        if let Some(file) = self.en_passant {
            self.hash ^= ZOBRIST.en_passant[file as usize];
        }
    }

    #[inline(always)]
    pub fn toggle_side_to_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.hash ^= ZOBRIST.black_to_move;
    }
}

#[cfg(test)]
mod tests {
    use crate::Board;

    #[test]
    fn zobrist_transpositions() {
        let board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
            .parse::<Board>().unwrap();
        const MOVES: &[[[&str; 4]; 2]] = &[
            [["e2c4", "h8f8", "d2h6", "b4b3"], ["e2c4", "b4b3", "d2h6", "h8f8"]],
            [["c3a4", "f6g8", "e1d1", "a8c8"], ["c3a4", "a8c8", "e1d1", "f6g8"]],
            [["h1g1", "f6g4", "d2h6", "b4b3"], ["h1g1", "b4b3", "d2h6", "f6g4"]],
            [["a1c1", "c7c5", "c3a4", "a6e2"], ["c3a4", "c7c5", "a1c1", "a6e2"]],
            [["e2c4", "h8h5", "f3f5", "e7d8"], ["f3f5", "h8h5", "e2c4", "e7d8"]],
            [["d5d6", "e8h8", "f3f6", "a6c4"], ["f3f6", "a6c4", "d5d6", "e8h8"]],
            [["f3e3", "e8h8", "a2a4", "a8c8"], ["a2a4", "a8c8", "f3e3", "e8h8"]],
            [["e1d1", "f6d5", "b2b3", "a8c8"], ["e1d1", "a8c8", "b2b3", "f6d5"]],
            [["e1d1", "e8f8", "e5c6", "h8h5"], ["e1d1", "h8h5", "e5c6", "e8f8"]],
            [["e2d3", "c7c6", "g2g4", "h8h6"], ["e2d3", "h8h6", "g2g4", "c7c6"]],
            [["f3h5", "f6h7", "c3b1", "g7f6"], ["c3b1", "f6h7", "f3h5", "g7f6"]],
            [["e2d3", "g6g5", "d2f4", "b6d5"], ["d2f4", "g6g5", "e2d3", "b6d5"]],
            [["a2a3", "h8h5", "c3b1", "a8d8"], ["a2a3", "a8d8", "c3b1", "h8h5"]],
            [["a2a4", "e8h8", "e1h1", "e7d8"], ["e1h1", "e8h8", "a2a4", "e7d8"]],
            [["b2b3", "e8f8", "g2g3", "a6b7"], ["b2b3", "a6b7", "g2g3", "e8f8"]],
            [["e5g4", "e8d8", "d2e3", "a6d3"], ["d2e3", "a6d3", "e5g4", "e8d8"]],
            [["g2h3", "e7d8", "e5g4", "b6c8"], ["e5g4", "b6c8", "g2h3", "e7d8"]],
            [["e5d3", "a6b7", "g2g3", "h8h6"], ["e5d3", "h8h6", "g2g3", "a6b7"]],
            [["e5g4", "h8h5", "f3f5", "e6f5"], ["f3f5", "e6f5", "e5g4", "h8h5"]],
            [["g2g3", "a8c8", "e5d3", "e7f8"], ["e5d3", "a8c8", "g2g3", "e7f8"]]
        ];
        for (i, [moves_a, moves_b]) in MOVES.iter().enumerate() {
            let mut board_a = board.clone();
            let mut board_b = board.clone();
            for mv in moves_a {
                board_a.play_unchecked(mv.parse().unwrap());
            }
            for mv in moves_b {
                board_b.play_unchecked(mv.parse().unwrap());
            }
            assert_eq!(board_a.hash(), board_b.hash(), "Test {}", i + 1);
        }
    }
}
