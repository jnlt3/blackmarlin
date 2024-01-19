use cozy_chess::{BitBoard, Board, Move, Piece};

#[test]
fn test_see() {
    use cozy_chess::Square;
    let fens = &[
        "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/3r4/3q4/3r4/8/3Q3K/3R4/7k w - - 0 1",
        "8/8/b7/1q6/2b5/3Q3K/4B3/7k w - - 0 1",
        "3r4/2P2n2/8/8/8/7K/8/7k w - - 0 1",
        "3r4/2P5/8/8/8/7K/8/7k w - - 0 1",
        "3R4/2P2n2/8/8/8/7K/8/7k b - - 0 1",
    ];
    let expected = &[
        piece_pts(Piece::Knight),
        piece_pts(Piece::Knight) - piece_pts(Piece::Rook),
        0,
        0,
        piece_pts(Piece::Rook) - piece_pts(Piece::Pawn),
        piece_pts(Piece::Rook) + piece_pts(Piece::Queen) - piece_pts(Piece::Pawn),
        piece_pts(Piece::Rook) + piece_pts(Piece::Pawn)
            - piece_pts(Piece::Knight)
            - piece_pts(Piece::Queen),
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

pub fn move_value(board: &Board, make_move: Move) -> i16 {
    board
        .piece_on(make_move.to)
        .map_or(0, |piece| piece_pts(piece))
}

pub fn mvv_lva(board: &Board, make_move: Move) -> i16 {
    let target = make_move.to;
    let piece = board.piece_on(make_move.from);

    let is_quiet = board.color_on(make_move.to) != Some(!board.side_to_move());

    let gain = match is_quiet {
        true => 0,
        false => match board.piece_on(target) {
            Some(piece) => piece_pts(piece),
            None => 0,
        },
    };
    if piece == Some(Piece::King) {
        return gain;
    }

    let blockers = board.occupied() & !make_move.from.bitboard();
    let stm = !board.side_to_move();
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

        if potential != BitBoard::EMPTY {
            let move_value = piece.map_or(0, piece_pts);
            return gain - move_value;
        }
    }
    0
}

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

fn piece_pts(piece: Piece) -> i16 {
    match piece {
        Piece::Pawn => 96,
        Piece::Knight => 323,
        Piece::Bishop => 323,
        Piece::Rook => 551,
        Piece::Queen => 864,
        Piece::King => i16::MAX / 2,
    }
}
