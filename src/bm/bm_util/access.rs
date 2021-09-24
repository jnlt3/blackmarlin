use chess::{BitBoard, Board, Color, Piece, EMPTY};

pub trait BbRepr {
    fn get_bit_board(board: &Board) -> BitBoard;
}
pub trait Attacker {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard;
}

impl<T: Attacker, U: Attacker> Attacker for (T, U) {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard {
        T::get_attacks(squares, blockers) | U::get_attacks(squares, blockers)
    }
}

pub struct And<T: BbRepr, U: BbRepr>(T, U);

pub struct Or<T: BbRepr, U: BbRepr>(T, U);

pub struct Not<T: BbRepr>(T);

pub struct AttacksOf<Squares: BbRepr, Blockers: BbRepr, Piece: Attacker>(Squares, Blockers, Piece);

impl<T: BbRepr, U: BbRepr> BbRepr for And<T, U> {
    fn get_bit_board(board: &Board) -> BitBoard {
        T::get_bit_board(board) & U::get_bit_board(board)
    }
}

impl<T: BbRepr, U: BbRepr> BbRepr for Or<T, U> {
    fn get_bit_board(board: &Board) -> BitBoard {
        T::get_bit_board(board) | U::get_bit_board(board)
    }
}

impl<T: BbRepr> BbRepr for Not<T> {
    fn get_bit_board(board: &Board) -> BitBoard {
        !T::get_bit_board(board)
    }
}

impl<Squares: BbRepr, Blockers: BbRepr, Piece: Attacker> BbRepr
    for AttacksOf<Squares, Blockers, Piece>
{
    #[inline]
    fn get_bit_board(board: &Board) -> BitBoard {
        Piece::get_attacks(
            Squares::get_bit_board(board),
            Blockers::get_bit_board(board),
        )
    }
}

macro_rules! impl_bb_type {
    ($name:ident, $board:ident, $operation:expr) => {
        pub struct $name {}

        impl BbRepr for $name {
            #[inline]
            fn get_bit_board($board: &Board) -> BitBoard {
                $operation
            }
        }
    };
}

impl_bb_type!(Full, _board, !EMPTY);
impl_bb_type!(Empty, _board, EMPTY);
impl_bb_type!(All, board, *board.combined());
impl_bb_type!(White, board, *board.color_combined(Color::White));
impl_bb_type!(Black, board, *board.color_combined(Color::Black));
impl_bb_type!(Pawns, board, *board.pieces(Piece::Pawn));
impl_bb_type!(Knights, board, *board.pieces(Piece::Knight));
impl_bb_type!(Bishops, board, *board.pieces(Piece::Bishop));
impl_bb_type!(Rooks, board, *board.pieces(Piece::Rook));
impl_bb_type!(Queens, board, *board.pieces(Piece::Queen));
impl_bb_type!(Kings, board, *board.pieces(Piece::King));

impl Attacker for (White, Pawns) {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_pawn_attacks(sq, Color::White, blockers);
        }
        attacks
    }
}

impl Attacker for (Black, Pawns) {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_pawn_attacks(sq, Color::Black, blockers);
        }
        attacks
    }
}

impl Attacker for Knights {
    fn get_attacks(squares: BitBoard, _: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_knight_moves(sq);
        }
        attacks
    }
}

impl Attacker for Bishops {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_bishop_moves(sq, blockers);
        }
        attacks
    }
}

impl Attacker for Rooks {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_rook_moves(sq, blockers);
        }
        attacks
    }
}

impl Attacker for Queens {
    fn get_attacks(squares: BitBoard, blockers: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_bishop_moves(sq, blockers) | chess::get_rook_moves(sq, blockers);
        }
        attacks
    }
}

impl Attacker for Kings {
    fn get_attacks(squares: BitBoard, _: BitBoard) -> BitBoard {
        let mut attacks = EMPTY;
        for sq in squares {
            attacks |= chess::get_king_moves(sq);
        }
        attacks
    }
}

macro_rules! attack_type {
    ($color:ident, $piece:ident) => {
        AttacksOf<And<$color, $piece>, All, $piece>
    }
}

#[macro_export]
macro_rules! access {
    (($($tail:tt)*)) => {
        access!($($tail)*)
    };
    (~$($tail:tt)*) => {
        Not<access!($($tail)*)>
    };
    ($head:ty, $($tail:tt)*) => {
        And<$head, access!($($tail)*)>
    };
    ($head:ty | $($tail:tt)*) => {
        Or<$head, access!($($tail)*)>
    };
    ($head:ty) => {
        $head
    };
}

pub type WhitePawnAttacks = AttacksOf<And<White, Pawns>, All, (White, Pawns)>;
pub type WhiteKnightAttacks = attack_type!(White, Knights);
pub type WhiteBishopAttacks = attack_type!(White, Bishops);
pub type WhiteRookAttacks = attack_type!(White, Rooks);
pub type WhiteQueenAttacks = attack_type!(White, Queens);
pub type WhiteKingAttacks = attack_type!(White, Kings);

pub type BlackPawnAttacks = AttacksOf<And<Black, Pawns>, All, (Black, Pawns)>;
pub type BlackKnightAttacks = attack_type!(Black, Knights);
pub type BlackBishopAttacks = attack_type!(Black, Bishops);
pub type BlackRookAttacks = attack_type!(Black, Rooks);
pub type BlackQueenAttacks = attack_type!(Black, Queens);
pub type BlackKingAttacks = attack_type!(Black, Kings);
