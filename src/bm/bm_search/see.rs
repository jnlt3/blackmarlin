use cozy_chess::{Board, Move, Piece};

pub fn calculate_see<const N: usize>(board: &Board, make_move: Move) -> i16 {
    let mut index = 0;
    let mut gains = [0_i16; N];
    let target_square = make_move.to;
    let mut move_piece = board.piece_on(make_move.from).unwrap();
    gains[0] = match board
        .piece_on(target_square)
        .zip(board.color_on(target_square))
    {
        Some((piece, color)) => match color == board.side_to_move() {
            true => return 0,
            false => piece_pts(piece),
        },
        None => match move_piece {
            Piece::King => return 0,
            _ => 0,
        },
    };
    let mut color = !board.side_to_move();
    let mut blockers = board.occupied() & !make_move.from.bitboard();

    let mut king_capture = false;

    'outer: for i in 1..N {
        gains[i] = piece_pts(move_piece) - gains[i - 1];
        if king_capture {
            index = i;
            break;
        }
        let defenders = board.colors(color) & blockers;
        for &piece in &Piece::ALL {
            let mut potential = match piece {
                Piece::Pawn => cozy_chess::get_pawn_attacks(target_square, !color),
                Piece::Knight => cozy_chess::get_knight_moves(target_square),
                Piece::Bishop => cozy_chess::get_bishop_moves(target_square, blockers),
                Piece::Rook => cozy_chess::get_rook_moves(target_square, blockers),
                Piece::Queen => {
                    cozy_chess::get_rook_moves(target_square, blockers)
                        | cozy_chess::get_bishop_moves(target_square, blockers)
                }
                Piece::King => cozy_chess::get_king_moves(target_square),
            } & board.pieces(piece)
                & defenders;
            if !potential.is_empty() {
                king_capture = move_piece == Piece::King;
                let attacker = potential.next().unwrap();
                blockers &= !attacker.bitboard();
                color = !color;
                move_piece = piece;
                continue 'outer;
            }
        }
        index = i;
        break;
    }
    for i in (1..index).rev() {
        gains[i - 1] = -i16::max(-gains[i - 1], gains[i]);
    }
    gains[0]
}

fn piece_pts(piece: Piece) -> i16 {
    match piece {
        Piece::Pawn => 96,
        Piece::Knight => 323,
        Piece::Bishop => 323,
        Piece::Rook => 551,
        Piece::Queen => 864,
        Piece::King => 20000,
    }
}
