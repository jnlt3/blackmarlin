use cozy_chess::{Board, Move, Piece};

#[test]
fn test_see() {
    use cozy_chess::Square;
    let fens = &[
        "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/3r4/3q4/3r4/8/3Q3K/3R4/7k w - - 0 1",
        "8/8/b7/1q6/2b5/3Q3K/4B3/7k w - - 0 1",
        "3r4/2P2n2/8/8/8/7K/8/7k w - - 0 1",
    ];
    let expected = &[
        piece_pts(Piece::Knight),
        piece_pts(Piece::Knight) - piece_pts(Piece::Rook),
        0,
        0,
        piece_pts(Piece::Rook) - piece_pts(Piece::Pawn),
    ];
    let mv_vert = Move {
        from: Square::D2,
        to: Square::D5,
        promotion: None,
    };
    let mv_diag = Move {
        from: Square::D3,
        to: Square::C4,
        promotion: None,
    };
    let cap_promo = Move {
        from: Square::C7,
        to: Square::D8,
        promotion: Some(Piece::Queen),
    };
    let k_cap = Move {
        from: Square::F7,
        to: Square::D8,
        promotion: None,
    };
    let moves = &[
        mv_vert, mv_vert, mv_vert, mv_diag, cap_promo, cap_promo, k_cap,
    ];

    for ((&fen, &expected), &mv) in fens.iter().zip(expected).zip(moves) {
        let board = Board::from_fen(fen, false).unwrap();
        assert!(compare_see(&board, mv, expected), "fen: {}", fen,);
        assert!(!compare_see(&board, mv, expected + 1), "fen: {}", fen,);
    }
}

/// Returns the value of the piece being captured
///
/// If there is no piece to capture, returns 0
pub fn move_value(board: &Board, make_move: Move) -> i16 {
    board
        .piece_on(make_move.to)
        .map_or(0, |piece| piece_pts(piece))
}

/// Returns true if SEE value is at least cmp
///
/// Will always prioritize the least valuable aggressor
///
/// Doesn't take promotions and pins into account
pub fn compare_see(board: &Board, make_move: Move, cmp: i16) -> bool {
    let target = make_move.to;
    let mut piece = board.piece_on(make_move.from);

    let is_quiet = board.color_on(make_move.to) != Some(!board.side_to_move());

    let mut gain = match is_quiet {
        true => 0,
        false => match board.piece_on(target) {
            Some(piece) => piece_pts(piece),
            None => 0,
        },
    };
    if piece == Some(Piece::King) {
        return gain >= cmp;
    }

    let mut blockers = board.occupied() & !make_move.from.bitboard();
    let mut stm = !board.side_to_move();
    'outer: for i in 1..16 {
        let start_stm = i % 2 == 0;

        if !start_stm && gain < cmp {
            return false;
        }
        if start_stm && gain >= cmp {
            return true;
        }

        for &attacker in &Piece::ALL {
            let pieces = board.colored_pieces(stm, attacker);
            if pieces.is_empty() {
                continue;
            }
            let potential = match attacker {
                Piece::Pawn => cozy_chess::get_pawn_attacks(target, !stm),
                Piece::Knight => cozy_chess::get_knight_moves(target),
                Piece::Bishop => cozy_chess::get_bishop_moves(target, blockers),
                Piece::Rook => cozy_chess::get_rook_moves(target, blockers),
                Piece::Queen => {
                    cozy_chess::get_rook_moves(target, blockers)
                        | cozy_chess::get_bishop_moves(target, blockers)
                }
                Piece::King => cozy_chess::get_king_moves(target),
            } & pieces
                & blockers;

            if let Some(sq) = potential.next_square() {
                blockers &= !sq.bitboard();
                let move_value = piece.map_or(0, piece_pts);
                match start_stm {
                    true => gain += move_value,
                    false => gain -= move_value,
                }
                piece = Some(attacker);
                stm = !stm;
                continue 'outer;
            }
        }
        break;
    }
    gain >= cmp
}

pub static mut PAWN: i16 = 96;
pub static mut MINOR: i16 = 323;
pub static mut ROOK: i16 = 551;
pub static mut QUEEN: i16 = 864;

/// Returns the piece values used by [compare_see](compare_see) and [move_value](move_value)
pub fn piece_pts(piece: Piece) -> i16 {
    unsafe {
        match piece {
            Piece::Pawn => PAWN,
            Piece::Knight => MINOR,
            Piece::Bishop => MINOR,
            Piece::Rook => ROOK,
            Piece::Queen => QUEEN,
            Piece::King => i16::MAX / 2,
        }
    }
}
