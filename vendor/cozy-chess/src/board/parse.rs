use core::convert::TryInto;
use core::str::FromStr;
use core::fmt::{Display, Formatter};

use crate::*;

use super::ZobristBoard;

helpers::simple_error! {
    /// An error while parsing the FEN.
    pub enum FenParseError {
        InvalidBoard = "The board is invalid.",
        InvalidSideToMove = "The side to move is invalid.",
        InvalidCastlingRights = "The castling rights are invalid.",
        InvalidEnPassant = "The en passant square is invalid.",
        InvalidHalfMoveClock = "The halfmove clock is invalid.",
        InvalidFullmoveNumber = "The fullmove number is invalid.",
        MissingField = "The FEN is missing a field.",
        TooManyFields = "The FEN has too many fields."
    }
}

impl Board {
    /// Parse a FEN string. If `shredder` is true, it parses Shredder FEN instead.
    /// You can also parse the board with [`FromStr`], which parses both FEN types.
    /// # Examples
    /// ## FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board = Board::from_fen(STARTPOS, false).unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    /// ## Shredder FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1";
    /// let board = Board::from_fen(STARTPOS, true).unwrap();
    /// assert_eq!(format!("{:#}", board), STARTPOS);
    /// ```
    pub fn from_fen(fen: &str, shredder: bool) -> Result<Self, FenParseError> {
        use FenParseError::*;

        let mut board = Self {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            halfmove_clock: 0,
            fullmove_number: 0
        };
        let mut parts = fen.split(' ');
        let mut next = || parts.next().ok_or(MissingField);
        Self::parse_board(&mut board, next()?)
            .map_err(|_| InvalidBoard)?;
        Self::parse_side_to_move(&mut board, next()?)
            .map_err(|_| InvalidSideToMove)?;
        if !board.board_is_valid() {
            return Err(InvalidBoard);
        }
        Self::parse_castle_rights(&mut board, next()?, shredder)
            .map_err(|_| InvalidCastlingRights)?;
        if !board.castle_rights_are_valid() {
            return Err(InvalidCastlingRights);
        }
        Self::parse_en_passant(&mut board, next()?)
            .map_err(|_| InvalidEnPassant)?;
        if !board.en_passant_is_valid() {
            return Err(InvalidEnPassant);
        }
        Self::parse_halfmove_clock(&mut board, next()?)
            .map_err(|_| InvalidHalfMoveClock)?;
        if !board.halfmove_clock_is_valid() {
            return Err(InvalidHalfMoveClock);
        }
        Self::parse_fullmove_number(&mut board, next()?)
            .map_err(|_| InvalidFullmoveNumber)?;
        if !board.fullmove_number_is_valid() {
            return Err(InvalidFullmoveNumber);
        }

        if parts.next().is_some() {
            return Err(TooManyFields);
        }

        let (checkers, pinned) = board.calculate_checkers_and_pins(board.side_to_move());
        board.checkers = checkers;
        board.pinned = pinned;
        if !board.checkers_and_pins_are_valid() {
            return Err(InvalidBoard);
        }

        Ok(board)
    }

    fn parse_board(board: &mut Board, s: &str) -> Result<(), ()> {
        for (rank, row) in s.rsplit('/').enumerate() {
            let rank = Rank::try_index(rank).ok_or(())?;
            let mut file = 0;
            for p in row.chars() {
                if let Some(offset) = p.to_digit(10) {
                    file += offset as usize;
                } else {
                    let piece = p.to_ascii_lowercase().try_into().map_err(|_| ())?;
                    let color = if p.is_ascii_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let square = Square::new(
                        File::try_index(file).ok_or(())?,
                        rank
                    );
                    board.inner.xor_square(piece, color, square);
                    file += 1;
                }
            }
            if file != File::NUM {
                return Err(());
            }
        }
        Ok(())
    }

    fn parse_side_to_move(board: &mut Board, s: &str) -> Result<(), ()> {
        if s.parse::<Color>().map_err(|_| ())? != board.side_to_move() {
            board.inner.toggle_side_to_move();
        }
        Ok(())
    }

    fn parse_castle_rights(board: &mut Board, s: &str, shredder: bool) -> Result<(), ()> {
        if s != "-" {
            for c in s.chars() {
                let color = if c.is_ascii_uppercase() {
                    Color::White
                } else {
                    Color::Black
                };
                let king_file = board.king(color).file();
                let (short, file) = if shredder {
                    let file = c.to_ascii_lowercase().try_into().map_err(|_| ())?;
                    (king_file < file, file)
                } else {
                    match c.to_ascii_lowercase() {
                        'k' => (true, File::H),
                        'q' => (false, File::A),
                        _ => return Err(())
                    }
                };
                let rights = board.castle_rights(color);
                let prev = if short {
                    rights.short
                } else {
                    rights.long
                };
                if prev.is_some() {
                    // Duplicates
                    return Err(());
                }
                board.inner.set_castle_right(color, short, Some(file));
            }
        }
        Ok(())
    }

    fn parse_en_passant(board: &mut Board, s: &str) -> Result<(), ()> {
        if s != "-" {
            let square = s.parse::<Square>().map_err(|_| ())?;
            let en_passant_rank = Rank::Third.relative_to(!board.side_to_move());
            if square.rank() != en_passant_rank {
                return Err(());
            }
            board.inner.set_en_passant(Some(square.file()));
        }
        Ok(())
    }

    fn parse_halfmove_clock(board: &mut Board, s: &str) -> Result<(), ()> {
        board.halfmove_clock = s.parse().map_err(|_| ())?;
        if board.halfmove_clock > 100 {
            return Err(());
        }
        Ok(())
    }

    fn parse_fullmove_number(board: &mut Board, s: &str) -> Result<(), ()> {
        board.fullmove_number = s.parse().map_err(|_| ())?;
        if board.fullmove_number == 0 {
            return Err(());
        }
        Ok(())
    }
}

impl FromStr for Board {
    type Err = FenParseError;

    /// Parse the board.
    /// This method will parse both regular FENs and Shredder FENs.
    /// See also: [`Board::from_fen`].
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board: Board = STARTPOS.parse().unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        match Self::from_fen(fen, false) {
            Ok(board) => Ok(board),
            Err(FenParseError::InvalidCastlingRights) => Self::from_fen(fen, true),
            Err(error) => Err(error)
        }
    }
}

impl Display for Board {
    /// Display the board. You can use the alternate format mode for Shredder FEN.
    /// # Examples
    /// ## FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board = Board::default();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    /// ## Shredder FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1";
    /// let board = Board::default();
    /// assert_eq!(format!("{:#}", board), STARTPOS);
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let shredder = f.alternate();
        for &rank in Rank::ALL.iter().rev() {
            let mut empty = 0;
            for &file in &File::ALL {
                let square = Square::new(file, rank);
                if let Some(piece) = self.piece_on(square) {
                    if empty > 0 {
                        write!(f, "{}", empty)?;
                        empty = 0;
                    }
                    let mut piece: char = piece.into();
                    if self.color_on(square).unwrap() == Color::White {
                        piece = piece.to_ascii_uppercase();
                    }
                    write!(f, "{}", piece)?;
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                write!(f, "{}", empty)?;
            }
            if rank > Rank::First {
                write!(f, "/")?;
            }
        }
        write!(f, " {} ", self.side_to_move())?;
        let mut wrote_castle_rights = false;
        for &color in &Color::ALL {
            let rights = self.castle_rights(color);
            let short = rights.short.map(|file| if shredder {
                file.into()
            } else {
                'k'
            });
            let long = rights.long.map(|file| if shredder {
                file.into()
            } else {
                'q'
            });
            for mut right in short.into_iter().chain(long) {
                if color == Color::White {
                    right = right.to_ascii_uppercase();
                }
                wrote_castle_rights = true;
                write!(f , "{}", right)?;
            }
        }
        if !wrote_castle_rights {
            write!(f , "-")?;
        }
        if let Some(file) = self.en_passant() {
            let rank = Rank::Third.relative_to(!self.side_to_move());
            write!(f, " {}", Square::new(file, rank))?;
        } else {
            write!(f, " -")?;
        }
        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_valid_fens() {
        for fen in include_str!("test_data/valid.sfens").lines() {
            let board = Board::from_fen(&fen, true).unwrap();
            assert!(board.validity_check());
        }
    }

    //No invalid FEN test yet due to lack of invalid FEN data.
}
