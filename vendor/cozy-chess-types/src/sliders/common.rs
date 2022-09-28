use crate::*;

pub const fn get_rook_relevant_blockers(square: Square) -> BitBoard {
    let rank_moves = square.rank().bitboard().0
        & !(File::A.bitboard().0 | File::H.bitboard().0);
    let file_moves = square.file().bitboard().0
        & !(Rank::First.bitboard().0 | Rank::Eighth.bitboard().0);
    BitBoard((rank_moves | file_moves) & !square.bitboard().0)
}

pub const fn get_bishop_relevant_blockers(square: Square) -> BitBoard {
    let mut rays = BitBoard::EMPTY.0;
    let mut i = 0;
    while i < Square::NUM {
        let target = Square::index_const(i);
        let rd = (square.rank() as i8 - target.rank() as i8).abs();
        let fd = (square.file() as i8 - target.file() as i8).abs();
        if rd == fd && rd != 0 {
            rays |= 1 << i;
        }
        i += 1;
    }
    BitBoard(rays & !BitBoard::EDGES.0)
}

const fn get_slider_moves(square: Square, mut blockers: BitBoard, deltas: &[(i8, i8); 4]) -> BitBoard {
    blockers.0 &= !square.bitboard().0;
    let mut moves = BitBoard::EMPTY;
    let mut i = 0;
    while i < deltas.len() {
        let (dx, dy) = deltas[i];
        let mut square = square;
        while !blockers.has(square) {
            if let Some(sq) = square.try_offset(dx, dy) {
                square = sq;
                moves.0 |= square.bitboard().0;
            } else {
                break;
            }
        }
        i += 1;
    }
    moves
}

pub const fn get_rook_moves_slow(square: Square, blockers: BitBoard) -> BitBoard {
    get_slider_moves(square, blockers, &[(1, 0), (0, -1), (-1, 0), (0, 1)])
}

pub const fn get_bishop_moves_slow(square: Square, blockers: BitBoard) -> BitBoard {
    get_slider_moves(square, blockers, &[(1, 1), (1, -1), (-1, -1), (-1, 1)])
}
