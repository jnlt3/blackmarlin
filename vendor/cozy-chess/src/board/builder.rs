use crate::*;

use super::zobrist::ZobristBoard;

helpers::simple_error! {
    /// An error while building a board.
    pub enum BoardBuilderError {
        InvalidBoard = "The board is invalid.",
        InvalidCastlingRights = "The castling rights are invalid.",
        InvalidEnPassant = "The en passant square is invalid.",
        InvalidHalfMoveClock = "The halfmove clock is invalid.",
        InvalidFullmoveNumber = "The fullmove number is invalid."
    }
}

/// A board builder to manipulate arbitrary boards.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoardBuilder {
    /// The board state. Index by square to get the corresponding piece.
    pub board: [Option<(Piece, Color)>; Square::NUM],
    /// The side to move.
    pub side_to_move: Color,
    /// The castling rights. Index by color to get the corresponding side's rights.
    pub castle_rights: [CastleRights; Color::NUM],
    /// The en passant square.
    pub en_passant: Option<Square>,
    /// The halfmove clock.
    pub halfmove_clock: u8,
    /// The fullmove number.
    pub fullmove_number: u16
}

impl Default for BoardBuilder {
    fn default() -> Self {
        BoardBuilder::startpos()
    }
}

impl BoardBuilder {
    /// Get an empty builder. All fields are set to their empty values.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::empty();
    /// for &square in &Square::ALL {
    ///     assert!(builder.square(square).is_none());
    /// }
    /// ```
    pub fn empty() -> Self {
        Self {
            board: [None; Square::NUM],
            side_to_move: Color::White,
            castle_rights: [CastleRights::EMPTY; Color::NUM],
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1
        }
    }

    /// Get a builder set to the default start position.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let startpos = Board::default();
    /// let builder = BoardBuilder::default();
    /// assert_eq!(builder.build().unwrap(), startpos);
    /// ```
    pub fn startpos() -> Self {
        Self::chess960_startpos(518)
    }

    /// Get a builder set to a chess960 start position.
    /// Converts a [scharnagl number](https://en.wikipedia.org/wiki/Fischer_random_chess_numbering_scheme)
    /// to its corresponding position.
    /// # Panics
    /// Panic if the scharnagl number is invalid (not within the range 0..960).
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let startpos = Board::default();
    /// // 518 is the scharnagl number for the default start position.
    /// let builder = BoardBuilder::chess960_startpos(518);
    /// assert_eq!(builder.build().unwrap(), startpos);
    /// ```
    pub fn chess960_startpos(scharnagl_number: u32) -> Self {
        Self::double_chess960_startpos(scharnagl_number, scharnagl_number)
    }

    /// Get a builder set to a double chess960 start position.
    /// Uses two [scharnagl numbers](https://en.wikipedia.org/wiki/Fischer_random_chess_numbering_scheme)
    /// for the initial setup for white and the initial setup for black.
    /// # Panics
    /// Panic if either scharnagl number is invalid (not within the range 0..960).
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let startpos = Board::default();
    /// // 518 is the scharnagl number for the default start position.
    /// let builder = BoardBuilder::double_chess960_startpos(518, 518);
    /// assert_eq!(builder.build().unwrap(), startpos);
    /// ```
    pub fn double_chess960_startpos(white_scharnagl_number: u32, black_scharnagl_number: u32) -> Self {
        let mut this = Self::empty();
        this.write_piece_config(white_scharnagl_number, Color::White);
        this.write_piece_config(black_scharnagl_number, Color::Black);
        this
    }

    fn write_piece_config(&mut self, scharnagl_number: u32, color: Color) {
        assert!(scharnagl_number < 960, "Scharnagl number must be in range 0..960");
        
        let n = scharnagl_number;
        let (n, light_bishop) = (n / 4, n % 4);
        let (n, dark_bishop) = (n / 4, n % 4);
        let (n, queen) = (n / 6, n % 6);
        let knights = n;

        let back_rank = Rank::First.relative_to(color);

        let mut free_squares = back_rank.bitboard();

        let light_bishop = match light_bishop {
            0 => File::B,
            1 => File::D,
            2 => File::F,
            3 => File::H,
            _ => unreachable!()
        };
        let light_bishop = Square::new(light_bishop, back_rank);
        free_squares ^= light_bishop.bitboard();

        let dark_bishop = match dark_bishop {
            0 => File::A,
            1 => File::C,
            2 => File::E,
            3 => File::G,
            _ => unreachable!()
        };
        let dark_bishop = Square::new(dark_bishop, back_rank);
        free_squares ^= dark_bishop.bitboard();

        let queen = free_squares.iter().nth(queen as usize).unwrap();
        free_squares ^= queen.bitboard();

        let (left_knight, right_knight) = match knights {
            0 => (0, 1),
            1 => (0, 2),
            2 => (0, 3),
            3 => (0, 4),

            4 => (1, 2),
            5 => (1, 3),
            6 => (1, 4),

            7 => (2, 3),
            8 => (2, 4),
            
            9 => (3, 4),

            _ => unreachable!()
        };
        let left_knight = free_squares.iter().nth(left_knight).unwrap();
        let right_knight = free_squares.iter().nth(right_knight).unwrap();
        free_squares ^= left_knight.bitboard();
        free_squares ^= right_knight.bitboard();

        let left_rook = free_squares.next_square().unwrap();
        free_squares ^= left_rook.bitboard();

        let king = free_squares.next_square().unwrap();
        free_squares ^= king.bitboard();

        let right_rook = free_squares.next_square().unwrap();
        free_squares ^= right_rook.bitboard();

        *self.square_mut(light_bishop) = Some((Piece::Bishop, color));
        *self.square_mut(dark_bishop)  = Some((Piece::Bishop, color));
        *self.square_mut(queen)        = Some((Piece::Queen, color));
        *self.square_mut(left_knight)  = Some((Piece::Knight, color));
        *self.square_mut(right_knight) = Some((Piece::Knight, color));
        *self.square_mut(left_rook)    = Some((Piece::Rook, color));
        *self.square_mut(king)         = Some((Piece::King, color));
        *self.square_mut(right_rook)   = Some((Piece::Rook, color));

        let pawn_rank = Rank::Second.relative_to(color);
        for square in pawn_rank.bitboard() {
            *self.square_mut(square) = Some((Piece::Pawn, color));
        }

        *self.castle_rights_mut(color) = CastleRights {
            short: Some(right_rook.file()),
            long: Some(left_rook.file())
        };
    }

    /// Create a builder from a [`Board`].
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// let builder = BoardBuilder::from_board(&board);
    /// assert_eq!(builder.build().unwrap(), board);
    /// ```
    pub fn from_board(board: &Board) -> Self {
        let mut this = BoardBuilder::empty();
        for &color in &Color::ALL {
            let pieces = board.colors(color);
            for &piece in &Piece::ALL {
                let pieces = pieces & board.pieces(piece);
                for square in pieces {
                    *this.square_mut(square) = Some((piece, color));
                }
            }
            *this.castle_rights_mut(color) = *board.castle_rights(color);
        }
        this.side_to_move = board.side_to_move();
        let en_passant_rank = Rank::Third.relative_to(!board.side_to_move());
        this.en_passant = board.en_passant().map(|f| Square::new(f, en_passant_rank));
        this.halfmove_clock = board.halfmove_clock();
        this.fullmove_number = board.fullmove_number();
        this
    }

    /// Get a square on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default();
    /// assert_eq!(builder.square(Square::A1), Some((Piece::Rook, Color::White)));
    /// ```
    pub fn square(&self, square: Square) -> Option<(Piece, Color)> {
        self.board[square as usize]
    }

    /// Mutably get a square on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut builder = BoardBuilder::default();
    /// *builder.square_mut(Square::A1) = Some((Piece::Knight, Color::White));
    /// assert_eq!(builder.square(Square::A1), Some((Piece::Knight, Color::White)));
    /// ```
    pub fn square_mut(&mut self, square: Square) -> &mut Option<(Piece, Color)> {
        &mut self.board[square as usize]
    }

    /// Get the castle rights for a side.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default();
    /// let rights = builder.castle_rights(Color::White);
    /// assert_eq!(rights.short, Some(File::H));
    /// assert_eq!(rights.long, Some(File::A));
    /// ```
    pub fn castle_rights(&self, color: Color) -> &CastleRights {
        &self.castle_rights[color as usize]
    }

    /// Mutably get the castle rights for a side.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut builder = BoardBuilder::default();
    /// let rights = builder.castle_rights_mut(Color::White);
    /// rights.short = None;
    /// assert_eq!(rights.short, None);
    /// ```
    pub fn castle_rights_mut(&mut self, color: Color) -> &mut CastleRights {
        &mut self.castle_rights[color as usize]
    }

    /// Build a [`Board`] from this builder.
    /// # Errors
    /// This will error if the current state is invalid.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default().build().unwrap();
    /// assert_eq!(builder, Board::default());
    /// ```
    pub fn build(&self) -> Result<Board, BoardBuilderError> {
        use BoardBuilderError::*;

        let mut board = Board {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            halfmove_clock: 0,
            fullmove_number: 0
        };

        self.add_board          (&mut board).map_err(|_| InvalidBoard)?;
        self.add_castle_rights  (&mut board).map_err(|_| InvalidCastlingRights)?;
        self.add_en_passant     (&mut board).map_err(|_| InvalidEnPassant)?;
        self.add_halfmove_clock (&mut board).map_err(|_| InvalidHalfMoveClock)?;
        self.add_fullmove_number(&mut board).map_err(|_| InvalidFullmoveNumber)?;
        
        let (checkers, pinned) = board.calculate_checkers_and_pins(board.side_to_move());
        board.checkers = checkers;
        board.pinned = pinned;
        Ok(board)
    }

    fn add_board(&self, board: &mut Board) -> Result<(), ()> {
        for &square in &Square::ALL {
            if let Some((piece, color)) = self.square(square) {
                board.inner.xor_square(piece, color, square);
            }
        }
        if self.side_to_move != board.side_to_move() {
            board.inner.toggle_side_to_move();
        }
        if !board.board_is_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_castle_rights(&self, board: &mut Board) -> Result<(), ()> {
        for &color in &Color::ALL {
            let rights = self.castle_rights[color as usize];
            board.inner.set_castle_right(color, true, rights.short);
            board.inner.set_castle_right(color, false, rights.long);
        }
        if !board.castle_rights_are_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_en_passant(&self, board: &mut Board) -> Result<(), ()> {
        if let Some(square) = self.en_passant {
            let en_passant_rank = Rank::Third.relative_to(!board.side_to_move());
            if square.rank() != en_passant_rank {
                return Err(());
            }
            board.inner.set_en_passant(Some(square.file()));
        }
        if !board.en_passant_is_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_halfmove_clock(&self, board: &mut Board) -> Result<(), ()> {
        if self.halfmove_clock > 100 {
            return Err(());
        }
        board.halfmove_clock = self.halfmove_clock;
        if !board.halfmove_clock_is_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_fullmove_number(&self, board: &mut Board) -> Result<(), ()> {
        board.fullmove_number = self.fullmove_number;
        if !board.fullmove_number_is_valid() {
            return Err(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_board() {
        let positions = include_str!("test_data/valid.sfens");
        for fen in positions.lines() {
            let board = Board::from_fen(fen, true).unwrap();
            let builder = BoardBuilder::from_board(&board);
            assert_eq!(builder.build().unwrap(), board);
        }
    }

    #[test]
    fn scharnagl_to_board() {
        let positions = include_str!("test_data/chess960_start_positions.sfens");
        for (scharnagl_number, fen) in positions.lines().enumerate() {
            let board = Board::from_fen(fen, true).unwrap();
            let builder = BoardBuilder::chess960_startpos(scharnagl_number as u32);
            assert_eq!(builder.build().unwrap(), board);
        }
    }

    // No invalid FEN test yet due to lack of invalid FEN data.
}
