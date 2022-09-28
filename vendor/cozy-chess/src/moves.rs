use crate::*;

include!(concat!(env!("OUT_DIR"), "/sliding_moves.rs"));

/// Get the moves for a rook on some square.
/// See [`get_rook_moves_const`] for a significantly slower `const` variant.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let blockers = bitboard! {
///     . . . X . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . X
///     . . . . . X . .
///     . . . . . . . .
/// };
/// let moves = get_rook_moves(Square::D3, blockers);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     X X X . X X X X
///     . . . X . . . .
///     . . . X . . . .
/// });
/// ```
#[inline(always)]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    BitBoard(SLIDING_MOVES[get_rook_moves_index(square, blockers)])
}

/// Significantly slower `const` version of [`get_rook_moves`].
pub const fn get_rook_moves_const(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_moves_slow(square, blockers)
}

/// Get the moves for a bishop on some square.
/// See [`get_bishop_moves_const`] for a significantly slower `const` variant.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let blockers = bitboard! {
///     . . . . . . . .
///     . . . . . . . X
///     . . X . . . . .
///     . . . . . X . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . X . .
/// };
/// let moves = get_bishop_moves(Square::D3, blockers);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     X . . . . . . .
///     . X . . . X . .
///     . . X . X . . .
///     . . . . . . . .
///     . . X . X . . .
///     . X . . . X . .
/// });
/// ```
#[inline(always)]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    BitBoard(SLIDING_MOVES[get_bishop_moves_index(square, blockers)])
}

/// Significantly slower `const` version of [`get_bishop_moves`].
pub const fn get_bishop_moves_const(square: Square, blockers: BitBoard) -> BitBoard {
    get_bishop_moves_slow(square, blockers)
}

/// Get the rays for a rook on some square.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let rays = get_rook_rays(Square::D3);
/// assert_eq!(rays, bitboard! {
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     X X X . X X X X
///     . . . X . . . .
///     . . . X . . . .
/// });
/// ```
#[inline(always)]
pub const fn get_rook_rays(square: Square) -> BitBoard {
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let square = Square::index_const(i);
            table[i] = get_rook_moves_const(square, BitBoard::EMPTY);
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get the rays for a bishop on some square.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let rays = get_bishop_rays(Square::D3);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . .
///     . . . . . . . X
///     X . . . . . X .
///     . X . . . X . .
///     . . X . X . . .
///     . . . . . . . .
///     . . X . X . . .
///     . X . . . X . .
/// });
/// ```
#[inline(always)]
pub const fn get_bishop_rays(square: Square) -> BitBoard {
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let square = Square::index_const(i);
            table[i] = get_bishop_moves_const(square, BitBoard::EMPTY);
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get all squares between two squares, if reachable via a ray.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let rays = get_between_rays(Square::B4, Square::G4);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X X X X . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// ```
#[inline(always)]
pub const fn get_between_rays(from: Square, to: Square) -> BitBoard {
    const fn get_between_rays(from: Square, to: Square) -> BitBoard {
        let dx = to.file() as i8 - from.file() as i8;
        let dy = to.rank() as i8 - from.rank() as i8;
        let orthogonal = dx == 0 || dy == 0;
        let diagonal = dx.abs() == dy.abs();
        if !(orthogonal || diagonal) {
            return BitBoard::EMPTY;
        }
        let dx = dx.signum();
        let dy = dy.signum();
        let mut square = from.offset(dx, dy);
        let mut between = BitBoard::EMPTY;
        while square as u8 != to as u8 {
            between.0 |= square.bitboard().0;
            square = square.offset(dx, dy);
        }
        between
    }
    const TABLE: [[BitBoard; Square::NUM]; Square::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let mut j = 0;
            while j < table[i].len() {
                table[i][j] = get_between_rays(
                    Square::index_const(i),
                    Square::index_const(j)
                );
                j += 1;
            }
            i += 1;
        }
        table
    };
    TABLE[from as usize][to as usize]
}

/// Get a ray on the board that passes through both squares, if it exists.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let rays = get_line_rays(Square::D2, Square::G5);
/// assert_eq!(rays, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . X
///     . . . . . . X .
///     . . . . . X . .
///     . . . . X . . .
///     . . . X . . . .
///     . . X . . . . .
/// });
/// ```
#[inline(always)]
pub const fn get_line_rays(from: Square, to: Square) -> BitBoard {
    const fn get_line_rays(from: Square, to: Square) -> BitBoard {
        let rays = get_bishop_rays(from);
        if rays.has(to) {
            return BitBoard((rays.0 | from.bitboard().0) & (get_bishop_rays(to).0 | to.bitboard().0));
        }
        let rays = get_rook_rays(from);
        if rays.has(to) {
            return BitBoard((rays.0 | from.bitboard().0) & (get_rook_rays(to).0 | to.bitboard().0));
        }
        BitBoard::EMPTY
    }
    const TABLE: [[BitBoard; Square::NUM]; Square::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            let mut j = 0;
            while j < table[i].len() {
                table[i][j] = get_line_rays(
                    Square::index_const(i),
                    Square::index_const(j)
                );
                j += 1;
            }
            i += 1;
        }
        table
    };
    TABLE[from as usize][to as usize]
}

/// Get the knight moves for a knight on some square.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let moves = get_knight_moves(Square::D3);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X . X . . .
///     . X . . . X . .
///     . . . . . . . .
///     . X . . . X . .
///     . . X . X . . .
/// });
/// ```
#[inline(always)]
pub const fn get_knight_moves(square: Square) -> BitBoard {
    const fn get_knight_moves(square: Square) -> BitBoard {
        const KNIGHT_DELTAS: [(i8, i8); 8] = [
            (-1, 2),
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1)
        ];
        let mut moves = BitBoard::EMPTY;
        let mut i = 0;
        while i < KNIGHT_DELTAS.len() {
            let (df, dr) = KNIGHT_DELTAS[i];
            if let Some(square) = square.try_offset(df, dr) {
                moves.0 |= square.bitboard().0;
            }
            i += 1;
        }
        moves
    }
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            table[i] = get_knight_moves(Square::index_const(i));
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get the king moves for a king on some square.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let moves = get_king_moves(Square::D3);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X X X . . .
///     . . X . X . . .
///     . . X X X . . .
///     . . . . . . . .
/// });
/// ```
#[inline(always)]
pub const fn get_king_moves(square: Square) -> BitBoard {
    const fn get_king_moves(square: Square) -> BitBoard {
        const KING_DELTAS: [(i8, i8); 8] = [
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1)
        ];
        let mut moves = BitBoard::EMPTY;
        let mut i = 0;
        while i < KING_DELTAS.len() {
            let (df, dr) = KING_DELTAS[i];
            if let Some(square) = square.try_offset(df, dr) {
                moves.0 |= square.bitboard().0;
            }
            i += 1;
        }
        moves
    }
    const TABLE: [BitBoard; Square::NUM] = {
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut i = 0;
        while i < table.len() {
            table[i] = get_king_moves(Square::index_const(i));
            i += 1;
        }
        table
    };
    TABLE[square as usize]
}

/// Get the pawn attacks for a pawn on some square.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let attacks = get_pawn_attacks(Square::D3, Color::White);
/// assert_eq!(attacks, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . X . X . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// ```
#[inline(always)]
pub const fn get_pawn_attacks(square: Square, color: Color) -> BitBoard {
    const fn get_pawn_attacks(square: Square, color: Color) -> BitBoard {
        const PAWN_DELTAS: [[(i8, i8); 2]; Color::NUM] = [
            [(1, 1), (-1, 1)],
            [(1, -1), (-1, -1)]
        ];
        let mut moves = BitBoard::EMPTY;
        let mut i = 0;
        while i < PAWN_DELTAS[color as usize].len() {
            let (df, dr) = PAWN_DELTAS[color as usize][i];
            if let Some(square) = square.try_offset(df, dr) {
                moves.0 |= square.bitboard().0;
            }
            i += 1;
        }
        moves
    }
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut c = 0;
        while c < table.len() {
            let mut i = 0;
            while i < table[c].len() {
                table[c][i] = get_pawn_attacks(
                    Square::index_const(i),
                    Color::index_const(c)
                );
                i += 1;
            }
            c += 1;
        }
        table
    };
    TABLE[color as usize][square as usize]
}

/// Get the pawn forward moves/non-captures for a pawn of some color on some square.
/// # Examples
/// ```
/// # use cozy_chess::*;
/// let moves = get_pawn_quiets(Square::D2, Color::White, BitBoard::EMPTY);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// 
/// let blockers = bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// };
/// let moves = get_pawn_quiets(Square::D7, Color::Black, blockers);
/// assert_eq!(moves, bitboard! {
///     . . . . . . . .
///     . . . . . . . .
///     . . . X . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
///     . . . . . . . .
/// });
/// ```
#[inline(always)]
pub const fn get_pawn_quiets(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    let square_bb = square.bitboard();
    let mut moves = BitBoard(if let Color::White = color {
        square_bb.0 << File::NUM
    } else {
        square_bb.0 >> File::NUM
    });
    moves.0 &= !blockers.0;
    if !moves.is_empty() && Rank::Second.relative_to(color).bitboard().has(square) {
        moves.0 |= if let Color::White = color {
            moves.0 << File::NUM
        } else {
            moves.0 >> File::NUM
        };
        moves.0 &= !blockers.0;
    }
    moves
}
