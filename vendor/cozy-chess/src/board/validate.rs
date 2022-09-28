use crate::*;

macro_rules! soft_assert {
    ($expr:expr) => {
        if !$expr {
            return false;
        }
    };
}

impl Board {
    /// Canonical implementation of board validity. Used for debugging.
    #[cfg(test)]
    pub(crate) fn validity_check(&self) -> bool {
        soft_assert!(self.board_is_valid());
        soft_assert!(self.castle_rights_are_valid());
        soft_assert!(self.en_passant_is_valid());
        soft_assert!(self.checkers_and_pins_are_valid());
        soft_assert!(self.halfmove_clock_is_valid());
        soft_assert!(self.fullmove_number_is_valid());
        true
    }

    /// Check if the just board is valid without considering "external" data like
    /// castle rights, en passant, or checker and pin info
    pub(super) fn board_is_valid(&self) -> bool {
        // Verify that the board's data makes sense. The bitboards should not overlap.
        let mut occupied = BitBoard::EMPTY;
        for &piece in &Piece::ALL {
            let pieces = self.pieces(piece);
            soft_assert!((pieces & occupied).is_empty());
            occupied |= pieces;
        }
        soft_assert!((self.colors(Color::White) & self.colors(Color::Black)).is_empty());
        soft_assert!(occupied == self.occupied());

        for &color in &Color::ALL {
            let pieces = self.colors(color);
            let no_pawn_mask = Rank::First.bitboard() | Rank::Eighth.bitboard();
            soft_assert!(pieces.len() <= 16);
            soft_assert!((pieces & self.pieces(Piece::King)).len() == 1);
            soft_assert!((pieces & self.pieces(Piece::Pawn)).len() <= 8);
            soft_assert!((pieces & self.pieces(Piece::Pawn) & no_pawn_mask).is_empty());
        }
        
        let (our_checkers, _) = self.calculate_checkers_and_pins(!self.side_to_move());
        // Opponent can't be in check while it's our turn
        soft_assert!(our_checkers.is_empty());

        true
    }

    pub(super) fn castle_rights_are_valid(&self) -> bool {
        for &color in &Color::ALL {
            let back_rank = Rank::First.relative_to(color);
            let rights = self.castle_rights(color);
            let our_rooks = self.colors(color) & self.pieces(Piece::Rook);
            if rights.short.is_some() || rights.long.is_some() {
                let our_king = self.king(color);
                soft_assert!(our_king.rank() == back_rank);
                if let Some(rook) = rights.long {
                    soft_assert!(our_rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(rook < our_king.file());
                }
                if let Some(rook) = rights.short {
                    soft_assert!(our_rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(our_king.file() < rook);
                }
            }
        }
        true
    }

    pub(super) fn en_passant_is_valid(&self) -> bool {
        let color = self.side_to_move();
        if let Some(en_passant) = self.en_passant() {
            let enemy_pawns = self.colors(!color) & self.pieces(Piece::Pawn);
            let en_passant_square = Square::new(
                en_passant,
                Rank::Third.relative_to(!color)
            );
            let en_passant_pawn = Square::new(
                en_passant,
                Rank::Fourth.relative_to(!color)
            );
            soft_assert!(!self.occupied().has(en_passant_square));
            soft_assert!(enemy_pawns.has(en_passant_pawn));
        }
        true
    }

    pub(super) fn checkers_and_pins_are_valid(&self) -> bool {
        let (checkers, pinned) = self.calculate_checkers_and_pins(self.side_to_move());
        soft_assert!(self.checkers() == checkers);
        soft_assert!(self.pinned() == pinned);
        soft_assert!(self.checkers().len() < 3);
        true
    }

    pub(super) fn halfmove_clock_is_valid(&self) -> bool {
        self.halfmove_clock <= 100
    }

    pub(super) fn fullmove_number_is_valid(&self) -> bool {
        self.fullmove_number > 0
    }

    pub(super) fn calculate_checkers_and_pins(&self, color: Color) -> (BitBoard, BitBoard) {
        let our_king = self.king(color);
        let their_pieces = self.colors(!color);

        let mut checkers = BitBoard::EMPTY;
        let mut pinned = BitBoard::EMPTY;

        let their_attackers = their_pieces & (
            (get_bishop_rays(our_king) & (
                self.pieces(Piece::Bishop) |
                self.pieces(Piece::Queen)
            )) |
            (get_rook_rays(our_king) & (
                self.pieces(Piece::Rook) |
                self.pieces(Piece::Queen)
            ))
        );
        for attacker in their_attackers {
            let between = get_between_rays(attacker, our_king) &
                self.occupied();
            match between.len() {
                0 => checkers |= attacker.bitboard(),
                1 => pinned |= between,
                _ => {}
            }
        }

        checkers |= get_knight_moves(our_king)
            & their_pieces
            & self.pieces(Piece::Knight);
        checkers |= get_pawn_attacks(our_king, color)
            & their_pieces
            & self.pieces(Piece::Pawn);
        (checkers, pinned)
    }
}
