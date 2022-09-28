use crate::*;

use super::*;

mod piece_moves;

pub use piece_moves::*;

#[cfg(test)]
mod tests;

mod slider {
    use super::*;

    pub trait SlidingPiece {
        const PIECE: Piece;

        fn pseudo_legals(square: Square, blockers: BitBoard) -> BitBoard;
    }

    macro_rules! impl_sliding_piece {
        ($square:ident,$color:ident,$blockers:ident; $($type:ident => $impl:expr),*) => {
            $(pub struct $type;

            impl SlidingPiece for $type {
                const PIECE: Piece = Piece::$type;

                fn pseudo_legals($square: Square, $blockers: BitBoard) -> BitBoard {
                    $impl
                }
            })*
        };
    }

    impl_sliding_piece! {
        square, color, blockers;
        Bishop => get_bishop_moves(square, blockers),
        Rook => get_rook_moves(square, blockers),
        Queen => get_bishop_moves(square, blockers) | get_rook_moves(square, blockers)
    }
}

macro_rules! abort_if {
    ($($expr:expr),*) => {
        $(if $expr {
            return true;
        })*
    }
}

impl Board {
    // Squares we can land on. When we're in check, we have to block
    // or capture the checker. In any case, we can't land on our own
    // pieces. Assumed to only be called if there is only one checker.
    fn target_squares<const IN_CHECK: bool>(&self) -> BitBoard {
        let color = self.side_to_move();
        let targets = if IN_CHECK {
            let checker = self.checkers().next_square().unwrap();
            let our_king = self.king(color);
            get_between_rays(checker, our_king) | checker.bitboard()
        } else {
            !BitBoard::EMPTY
        };
        targets & !self.colors(color)
    }

    fn add_slider_legals<
        P: slider::SlidingPiece, F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: BitBoard, listener: &mut F) -> bool {
        let color = self.side_to_move();
        let our_king = self.king(color);
        let pieces = self.colored_pieces(color, P::PIECE) & mask;
        let pinned = self.pinned();
        let blockers = self.occupied();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = P::pseudo_legals(piece, blockers) & target_squares;
            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: P::PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }

        if !IN_CHECK {
            for piece in pieces & pinned {
                //If we're not in check, we can still slide along the pinned ray.
                let target_squares = target_squares & get_line_rays(our_king, piece);
                let moves = P::pseudo_legals(piece, blockers) & target_squares;
                if !moves.is_empty() {
                    abort_if!(listener(PieceMoves {
                        piece: P::PIECE,
                        from: piece,
                        to: moves
                    }));
                }
            }
        }
        false
    }

    fn add_knight_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: BitBoard, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::Knight;

        let color = self.side_to_move();
        let pieces = self.colored_pieces(color, PIECE) & mask;
        let pinned = self.pinned();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = get_knight_moves(piece) & target_squares;
            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }
        false
    }

    fn add_pawn_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: BitBoard, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::Pawn;

        let color = self.side_to_move();
        let our_king = self.king(color);
        let pieces = self.colored_pieces(color, PIECE) & mask;
        let their_pieces = self.colors(!color);
        let pinned = self.pinned();
        let blockers = self.occupied();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = (
                get_pawn_quiets(piece, color, blockers) |
                (get_pawn_attacks(piece, color) & their_pieces)
            ) & target_squares;
            if !moves.is_empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }

        if !IN_CHECK {
            for piece in pieces & pinned {
                //If we're not in check, we can still slide along the pinned ray.
                let target_squares = target_squares & get_line_rays(our_king, piece);
                let moves = (
                    get_pawn_quiets(piece, color, blockers) |
                    (get_pawn_attacks(piece, color) & their_pieces)
                ) & target_squares;
                if !moves.is_empty() {
                    abort_if!(listener(PieceMoves {
                        piece: PIECE,
                        from: piece,
                        to: moves
                    }));
                }
            }
        }

        if let Some(en_passant) = self.en_passant() {
            let their_bishops = their_pieces & (
                self.pieces(Piece::Bishop) |
                self.pieces(Piece::Queen)
            );
            let their_rooks = their_pieces & (
                self.pieces(Piece::Rook) |
                self.pieces(Piece::Queen)
            );

            let dest = Square::new(en_passant, Rank::Third.relative_to(!color));
            let victim = Square::new(en_passant, Rank::Fourth.relative_to(!color));
            for piece in get_pawn_attacks(dest, !color) & pieces {
                //Simulate the capture and update the pieces accordingly.
                let blockers = blockers
                    ^ victim.bitboard()
                    ^ piece.bitboard()
                    | dest.bitboard();
                //First test a basic ray to prevent an expensive magic lookup
                let on_ray = !(get_bishop_rays(our_king) & their_bishops).is_empty();
                if on_ray && !(get_bishop_moves(our_king, blockers) & their_bishops).is_empty() {
                    continue;
                }
                let on_ray = !(get_rook_rays(our_king) & their_rooks).is_empty();
                if on_ray && !(get_rook_moves(our_king, blockers) & their_rooks).is_empty() {
                    continue;
                }
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: dest.bitboard()
                }));
            }
        }
        false
    }

    #[inline(always)]
    fn king_safe_on(&self, square: Square) -> bool {
        macro_rules! short_circuit {
            ($($attackers:expr),*) => {
                $(if !$attackers.is_empty() {
                    return false;
                })*
                true
            }
        }

        let color = self.side_to_move();
        let their_pieces = self.colors(!color);
        let blockers = self.occupied()
            ^ self.colored_pieces(color, Piece::King)
            | square.bitboard();
        short_circuit! {
            get_bishop_moves(square, blockers) & their_pieces & (
                self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)
            ),
            get_rook_moves(square, blockers) & their_pieces & (
                self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
            ),
            get_knight_moves(square) & their_pieces & self.pieces(Piece::Knight),
            get_king_moves(square) & their_pieces & self.pieces(Piece::King),
            get_pawn_attacks(square, color) & their_pieces & self.pieces(Piece::Pawn)
        }
    }

    fn can_castle(&self, rook: File, king_dest: File, rook_dest: File) -> bool {
        let color = self.side_to_move();
        let our_king = self.king(color);
        let back_rank = Rank::First.relative_to(color);
        let blockers = self.occupied() ^ our_king.bitboard();
        let pinned = self.pinned();
        let rook = Square::new(rook, back_rank);
        let blockers = blockers ^ rook.bitboard();
        let king_dest = Square::new(king_dest, back_rank);
        let rook_dest = Square::new(rook_dest, back_rank);
        let king_to_rook = get_between_rays(our_king, rook);
        let king_to_dest = get_between_rays(our_king, king_dest);
        let must_be_safe = king_to_dest | king_dest.bitboard();
        let must_be_empty = must_be_safe | king_to_rook | rook_dest.bitboard();
        !pinned.has(rook)
            && (blockers & must_be_empty).is_empty()
            && must_be_safe.iter().all(|square| self.king_safe_on(square))
    }

    fn add_king_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: BitBoard, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::King;

        let color = self.side_to_move();
        let our_pieces = self.colors(color);
        let our_king = self.king(color);
        if !mask.has(our_king) {
            return false;
        }
        let mut moves = BitBoard::EMPTY;
        for to in get_king_moves(our_king) & !our_pieces {
            if self.king_safe_on(to) {
                moves |= to.bitboard();
            }
        }
        if !IN_CHECK {
            let rights = self.castle_rights(color);
            let back_rank = Rank::First.relative_to(color);
            if let Some(rook) = rights.short {
                if self.can_castle(rook, File::G, File::F) {
                    moves |= Square::new(rook, back_rank).bitboard();
                }
            }
            if let Some(rook) = rights.long {
                if self.can_castle(rook, File::C, File::D) {
                    moves |= Square::new(rook, back_rank).bitboard();
                }
            }
        }
        if !moves.is_empty() {
            abort_if!(listener(PieceMoves {
                piece: PIECE,
                from: our_king,
                to: moves
            }));
        }
        false
    }

    fn add_all_legals<
        F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, mask: BitBoard, listener: &mut F) -> bool {
        abort_if! {
            self.add_pawn_legals::<_, IN_CHECK>(mask, listener),
            self.add_knight_legals::<_, IN_CHECK>(mask, listener),
            self.add_slider_legals::<slider::Bishop, _, IN_CHECK>(mask, listener),
            self.add_slider_legals::<slider::Rook, _, IN_CHECK>(mask, listener),
            self.add_slider_legals::<slider::Queen, _, IN_CHECK>(mask, listener),
            self.add_king_legals::<_, IN_CHECK>(mask, listener)
        }
        false
    }

    /// Generate all legal moves given a position in no particular order.
    /// To retrieve the moves, a `listener` callback must be passed that receives compact [`PieceMoves`].
    /// This does *not* guarantee that each [`PieceMoves`] value has a unique `from` square.
    /// However, each [`PieceMoves`] value will have at least one move.
    /// The listener will be called a maximum of 18 times.
    /// The listener can abort the movegen early by returning `true`.
    /// In this case, this function also returns `true`.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// let mut total_moves = 0;
    /// board.generate_moves(|moves| {
    ///     // Done this way for demonstration.
    ///     // Actual counting is best done in bulk with moves.len().
    ///     for _mv in moves {
    ///         total_moves += 1;
    ///     }
    ///     false
    /// });
    /// assert_eq!(total_moves, 20);
    /// ```
    pub fn generate_moves(&self, listener: impl FnMut(PieceMoves) -> bool) -> bool {
        self.generate_moves_for(BitBoard::FULL, listener)
    }

    /// Version of [`Board::generate_moves`] moves that
    /// generates moves for only a subset of pieces.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// let knights = board.pieces(Piece::Knight);
    /// let mut knight_moves = 0;
    /// board.generate_moves_for(knights, |moves| {
    ///     // Done this way for demonstration.
    ///     // Actual counting is best done in bulk with moves.len().
    ///     for _mv in moves {
    ///         knight_moves += 1;
    ///     }
    ///     false
    /// });
    /// assert_eq!(knight_moves, 4);
    /// ```
    pub fn generate_moves_for(
        &self, mask: BitBoard, mut listener: impl FnMut(PieceMoves) -> bool
    ) -> bool {
        match self.checkers().len() {
            0 => self.add_all_legals::<_, false>(mask, &mut listener),
            1 => self.add_all_legals::<_, true>(mask, &mut listener),
            _ => self.add_king_legals::<_, true>(mask, &mut listener)
        }
    }

    fn king_is_legal(&self, mv: Move) -> bool {
        if self.checkers.is_empty() {
            let castles = self.castle_rights(self.side_to_move());
            let back_rank = Rank::First.relative_to(self.side_to_move());
            if let Some(rook) = castles.short {
                let rook_square = Square::new(rook, back_rank);
                if rook_square == mv.to && self.can_castle(rook, File::G, File::F) {
                    return true;
                }
            }
            if let Some(rook) = castles.long {
                let rook_square = Square::new(rook, back_rank);
                if rook_square == mv.to && self.can_castle(rook, File::C, File::D) {
                    return true;
                }
            }
        }
        if !(get_king_moves(mv.from) & !self.colors(self.side_to_move())).has(mv.to) {
            return false;
        }
        if mv.promotion.is_some() {
            return false;
        }
        self.king_safe_on(mv.to)
    }

    /// See if a move is legal.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// assert!(board.is_legal("e2e4".parse().unwrap()));
    /// assert!(!board.is_legal("e1e8".parse().unwrap()));
    /// ```
    pub fn is_legal(&self, mv: Move) -> bool {
        if !self.colors(self.side_to_move()).has(mv.from) {
            return false;
        }

        let king_sq = self.king(self.side_to_move());
        if mv.from == king_sq {
            if mv.promotion.is_some() {
                return false;
            }
            return self.king_is_legal(mv);
        }

        if self.pinned().has(mv.from) && !get_line_rays(king_sq, mv.from).has(mv.to) {
            return false;
        }

        let target_squares = match self.checkers().len() {
            0 => self.target_squares::<false>(),
            1 => self.target_squares::<true>(),
            _ => return false,
        };

        let piece = self.piece_on(mv.from);
        if piece != Some(Piece::Pawn) && mv.promotion.is_some() {
            return false;
        }

        match piece {
            None | Some(Piece::King) => false, // impossible
            Some(Piece::Pawn) => {
                let promo_rank = Rank::Eighth.relative_to(self.side_to_move());
                match (mv.to.rank() == promo_rank, mv.promotion) {
                    (true, Some(Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen)) => {}
                    (false, None) => {}
                    _ => return false,
                }
                let mut c = |moves: PieceMoves| moves.to.has(mv.to);
                if self.checkers().is_empty() {
                    self.add_pawn_legals::<_, false>(mv.from.bitboard(), &mut c)
                } else {
                    self.add_pawn_legals::<_, true>(mv.from.bitboard(), &mut c)
                }
            }
            Some(Piece::Rook) => {
                (target_squares & get_rook_rays(mv.from)).has(mv.to)
                    && (get_between_rays(mv.from, mv.to) & self.occupied()).is_empty()
            }
            Some(Piece::Bishop) => {
                (target_squares & get_bishop_rays(mv.from)).has(mv.to)
                    && (get_between_rays(mv.from, mv.to) & self.occupied()).is_empty()
            }
            Some(Piece::Knight) => (target_squares & get_knight_moves(mv.from)).has(mv.to),
            Some(Piece::Queen) => {
                (target_squares & (get_rook_rays(mv.from) | get_bishop_rays(mv.from))).has(mv.to)
                    && (get_between_rays(mv.from, mv.to) & self.occupied()).is_empty()
            }
        }
    }
}
