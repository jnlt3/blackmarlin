use chess::{BitBoard, Board, Color, Piece, EMPTY};

pub trait Access {
    fn get(resource: &EvalResource) -> BitBoard;
}

pub struct EvalResource<'a> {
    board: &'a Board,

    w_attack: BitBoard,
    b_attack: BitBoard,
}

impl<'a> EvalResource<'a> {
    pub fn new(board: &'a Board) -> Self {
        let mut w_non_king_att = EMPTY;
        let mut b_non_king_att = EMPTY;

        let white = *board.color_combined(Color::White);
        let black = *board.color_combined(Color::Black);
        let blockers = white | black;
        for sq in white & *board.pieces(Piece::Pawn) {
            w_non_king_att |= chess::get_pawn_attacks(sq, Color::White, !EMPTY)
        }
        for sq in white & *board.pieces(Piece::Knight) {
            w_non_king_att |= chess::get_knight_moves(sq);
        }
        for sq in white & (*board.pieces(Piece::Bishop) | *board.pieces(Piece::Queen)) {
            w_non_king_att |= chess::get_bishop_moves(sq, blockers);
        }
        for sq in white & (*board.pieces(Piece::Rook) | *board.pieces(Piece::Queen)) {
            w_non_king_att |= chess::get_rook_moves(sq, blockers);
        }
        for sq in white & *board.pieces(Piece::King) {
            w_non_king_att |= chess::get_king_moves(sq);
        }

        for sq in black & *board.pieces(Piece::Pawn) {
            b_non_king_att |= chess::get_pawn_attacks(sq, Color::Black, !EMPTY)
        }
        for sq in black & *board.pieces(Piece::Knight) {
            b_non_king_att |= chess::get_knight_moves(sq);
        }
        for sq in black & (*board.pieces(Piece::Bishop) | *board.pieces(Piece::Queen)) {
            b_non_king_att |= chess::get_bishop_moves(sq, blockers);
        }
        for sq in black & (*board.pieces(Piece::Rook) | *board.pieces(Piece::Queen)) {
            b_non_king_att |= chess::get_rook_moves(sq, blockers);
        }
        for sq in black & *board.pieces(Piece::King) {
            b_non_king_att |= chess::get_king_moves(sq);
        }

        Self {
            w_attack: w_non_king_att,
            b_attack: b_non_king_att,
            board,
        }
    }

    pub fn get<T: Access>(&self) -> BitBoard {
        T::get(&self)
    }
}

macro_rules! impl_access {
    ($name:ident, $res:ident, $func:expr) => {
        pub struct $name;

        impl Access for $name {
            fn get($res: &EvalResource) -> BitBoard {
                $func
            }
        }
    };
}

impl_access!(Pawns, res, *res.board.pieces(Piece::Pawn));
impl_access!(Knights, res, *res.board.pieces(Piece::Knight));
impl_access!(Bishops, res, *res.board.pieces(Piece::Bishop));
impl_access!(Rooks, res, *res.board.pieces(Piece::Rook));
impl_access!(Queens, res, *res.board.pieces(Piece::Queen));
impl_access!(Kings, res, *res.board.pieces(Piece::King));
impl_access!(White, res, *res.board.color_combined(Color::White));
impl_access!(Black, res, *res.board.color_combined(Color::Black));
impl_access!(All, res, *res.board.combined());
impl_access!(WhiteNonKingAttack, res, res.w_attack);
impl_access!(BlackNonKingAttack, res, res.b_attack);
