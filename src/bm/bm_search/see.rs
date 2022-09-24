use cozy_chess::{BitBoard, Board, Move, Piece};

pub fn calculate_see<const N: usize>(board: &Board, make_move: Move) -> i16 {
    let mut index = 0;
    let mut gains = [0_i16; N];
    let target_square = make_move.to;
    let move_piece = board.piece_on(make_move.from).unwrap();
    gains[0] = if let Some((piece, color)) = board
        .piece_on(target_square)
        .zip(board.color_on(target_square))
    {
        if color == board.side_to_move() {
            return 0;
        }
        piece_pts(piece)
    } else {
        if move_piece == Piece::King {
            return 0;
        }
        0
    };
    let mut color = !board.side_to_move();
    let mut blockers = board.occupied() & !make_move.from.bitboard();
    let mut last_piece_pts = piece_pts(move_piece);
    'outer: for i in 1..N {
        gains[i] = last_piece_pts - gains[i - 1];
        let defenders = board.colors(color) & blockers;
        for &piece in &Piece::ALL {
            last_piece_pts = piece_pts(piece);
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
            if potential != BitBoard::EMPTY {
                let attacker = potential.next().unwrap();
                blockers &= !attacker.bitboard();
                color = !color;
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
