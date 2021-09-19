use crate::bm::bm_eval::basic_eval_consts::*;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_util::evaluator::Evaluator;
use crate::bm::bm_util::position::Position;
use chess::{BitBoard, Board, ChessMove, Color, File, Piece, ALL_FILES, EMPTY};

const PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicEvalData {
    w_ahead: [BitBoard; 64],
    b_ahead: [BitBoard; 64],

    king_flank: [BitBoard; 8],
}

pub const fn get_basic_eval_data() -> BasicEvalData {
    let mut data = BasicEvalData {
        w_ahead: [BitBoard(0); 64],
        b_ahead: [BitBoard(0); 64],
        king_flank: [BitBoard(0); 8],
    };


    let mut king_flank = 0_u64;
    let mut queen_flank = 0_u64;
    let mut file = 0_u8;
    while file < 8 {
        let mut file_bb = 0u64;
        let mut rank = 0_u8;
        while rank < 8 {
            file_bb |= 1_u64 << (rank * 8 + file);
            rank += 1;
        }
        if file < 4 {
            king_flank |= file_bb;
        } else {
            queen_flank |= file_bb
        }
        file += 1;
    }

    let mut file = 0_u8;
    while file < 8 {
        data.king_flank[file as usize] = if file < 4 {
            BitBoard(king_flank)
        } else {
            BitBoard(queen_flank)
        };
        file += 1;
    }

    let mut king_rank = 0_u8;
    while king_rank < 8 {
        let mut king_file = 0_u8;
        while king_file < 8 {
            let king = (king_rank * 8 + king_file) as usize;
            let mut w_ahead = 0_u64;
            let mut b_ahead = 0_u64;
            {
                let king_rank = king_rank as i32;
                let king_file = king_file as i32;
                let mut rank = 0_u8;
                while rank < 8 {
                    let mut file = 0_u8;
                    while file < 8 {
                        let sq = rank * 8 + file;
                        {
                            let file = file as i32;
                            let rank = rank as i32;

                            let file_diff = (file - king_file).abs();
                            let rank_diff = rank - king_rank;

                            let bitboard = 1_u64 << sq;
                            if file_diff <= 1 && rank_diff > 0 {
                                w_ahead |= bitboard;
                            }
                            let rank_diff = king_rank - rank;
                            if file_diff <= 1 && rank_diff > 0 {
                                b_ahead |= bitboard
                            }
                        }
                        file += 1;
                    }
                    rank += 1;
                }
            }
            data.w_ahead[king] = BitBoard(w_ahead);
            data.b_ahead[king] = BitBoard(b_ahead);

            king_file += 1;
        }
        king_rank += 1;
    }
    data
}

const DATA: BasicEvalData = get_basic_eval_data();

#[derive(Debug, Clone)]
pub struct BasicEval;

impl Evaluator for BasicEval {
    fn new() -> Self {
        Self
    }

    fn see(mut board: Board, mut make_move: ChessMove) -> i32 {
        let mut index = 0;
        let mut gains = [0i32; 32];
        let target_square = make_move.get_dest();
        gains[0] = Self::piece_pts(board.piece_on(target_square).unwrap());
        'outer: for i in 1..32 {
            board = board.make_move_new(make_move);
            gains[i] = Self::piece_pts(board.piece_on(target_square).unwrap()) - gains[i - 1];
            let color = board.side_to_move();
            let defenders = *board.color_combined(color);
            let blockers = *board.combined();
            for piece in &PIECES {
                match piece {
                    Piece::Pawn => {
                        let mut potential =
                            chess::get_pawn_attacks(target_square, !color, blockers)
                                & defenders
                                & board.pieces(Piece::Pawn);
                        if potential != EMPTY {
                            let attacker = potential.next().unwrap();
                            make_move = ChessMove::new(attacker, target_square, None);
                            continue 'outer;
                        }
                    }
                    Piece::Knight => {
                        let mut potential = chess::get_knight_moves(target_square)
                            & board.pieces(Piece::Knight)
                            & defenders;
                        if potential != EMPTY {
                            let attacker = potential.next().unwrap();
                            make_move = ChessMove::new(attacker, target_square, None);
                            continue 'outer;
                        }
                    }
                    Piece::Bishop => {
                        let mut potential = chess::get_bishop_moves(target_square, blockers)
                            & defenders
                            & board.pieces(Piece::Bishop);
                        if potential != EMPTY {
                            let attacker = potential.next().unwrap();
                            make_move = ChessMove::new(attacker, target_square, None);
                            continue 'outer;
                        }
                    }
                    Piece::Rook => {
                        let mut potential = chess::get_rook_moves(target_square, blockers)
                            & board.pieces(Piece::Rook)
                            & defenders;
                        if potential != EMPTY {
                            let attacker = potential.next().unwrap();
                            make_move = ChessMove::new(attacker, target_square, None);
                            continue 'outer;
                        }
                    }
                    Piece::Queen => {
                        let mut potential = chess::get_rook_moves(target_square, blockers)
                            & chess::get_bishop_moves(target_square, blockers)
                            & board.pieces(Piece::Queen)
                            & defenders;
                        if potential != EMPTY {
                            let attacker = potential.next().unwrap();
                            make_move = ChessMove::new(attacker, target_square, None);
                            continue 'outer;
                        }
                    }
                    Piece::King => {
                        let mut potential = chess::get_king_moves(target_square)
                            & board.pieces(Piece::King)
                            & defenders;
                        if potential != EMPTY {
                            let attacker = potential.next().unwrap();
                            make_move = ChessMove::new(attacker, target_square, None);
                            continue 'outer;
                        }
                    }
                }
            }
            index = i;
            break;
        }
        for i in (1..index).rev() {
            gains[i - 1] = -i32::max(-gains[i - 1], gains[i]);
        }
        gains[0]
    }

    /**
    Doesn't handle checkmates or stalemates
     */
    fn evaluate(&mut self, position: &Position) -> Evaluation {
        let board = position.board();

        let turn = position.turn();

        let pawns = *board.pieces(Piece::Pawn);
        let knights = *board.pieces(Piece::Knight);
        let bishops = *board.pieces(Piece::Bishop);
        let rooks = *board.pieces(Piece::Rook);
        let queens = *board.pieces(Piece::Queen);
        let kings = *board.pieces(Piece::King);

        let phase = TOTAL_PHASE.saturating_sub(
            pawns.popcnt() * PAWN_PHASE
                + knights.popcnt() * KNIGHT_PHASE
                + bishops.popcnt() * BISHOP_PHASE
                + rooks.popcnt() * ROOK_PHASE
                + queens.popcnt() * QUEEN_PHASE,
        ) as i32;

        let white = *board.color_combined(Color::White);
        let black = *board.color_combined(Color::Black);

        let white_pawns = pawns & white;
        let white_knights = knights & white;
        let white_bishops = bishops & white;
        let white_rooks = rooks & white;
        let white_queens = queens & white;
        let white_king = kings & white;

        let black_pawns = pawns & black;
        let black_knights = knights & black;
        let black_bishops = bishops & black;
        let black_rooks = rooks & black;
        let black_queens = queens & black;
        let black_king = kings & black;

        let phase = phase as i32;

        //PSQT
        let white_psqt_score =
            Self::get_white_psqt_score(white_pawns, &PAWN_TABLE, &PAWN_END_TABLE, phase)
                + Self::get_white_psqt_score(
                    white_knights,
                    &KNIGHT_TABLE,
                    &KNIGHT_END_TABLE,
                    phase,
                )
                + Self::get_white_psqt_score(
                    white_bishops,
                    &BISHOP_TABLE,
                    &BISHOP_END_TABLE,
                    phase,
                )
                + Self::get_white_psqt_score(white_rooks, &ROOK_TABLE, &ROOK_END_TABLE, phase)
                + Self::get_white_psqt_score(white_queens, &QUEEN_TABLE, &QUEEN_END_TABLE, phase)
                + Self::get_white_psqt_score(white_king, &KING_TABLE, &KING_END_TABLE, phase);

        let black_psqt_score =
            Self::get_black_psqt_score(black_pawns, &PAWN_TABLE, &PAWN_END_TABLE, phase)
                + Self::get_black_psqt_score(
                    black_knights,
                    &KNIGHT_TABLE,
                    &KNIGHT_END_TABLE,
                    phase,
                )
                + Self::get_black_psqt_score(
                    black_bishops,
                    &BISHOP_TABLE,
                    &BISHOP_END_TABLE,
                    phase,
                )
                + Self::get_black_psqt_score(black_rooks, &ROOK_TABLE, &ROOK_END_TABLE, phase)
                + Self::get_black_psqt_score(black_queens, &QUEEN_TABLE, &QUEEN_END_TABLE, phase)
                + Self::get_black_psqt_score(black_king, &KING_TABLE, &KING_END_TABLE, phase);

        let psqt_score = white_psqt_score - black_psqt_score;

        let blockers = *board.combined();

        let mut white_attacked = EMPTY;
        let mut black_attacked = EMPTY;

        let mut w_pawn_attack = EMPTY;
        let mut b_pawn_attack = EMPTY;

        for pawn in white_pawns {
            let attacks = chess::get_pawn_attacks(pawn, Color::White, black);
            white_attacked |= attacks;
            w_pawn_attack |= attacks;
        }
        for knight in white_knights {
            let attacks = chess::get_knight_moves(knight);
            white_attacked |= attacks;
        }
        for bishop in white_bishops {
            let blockers = blockers & !white_bishops & !white_queens;
            let attacks = chess::get_bishop_moves(bishop, blockers);
            white_attacked |= attacks;
        }
        for rook in white_rooks {
            let blockers = blockers & !white_rooks & !white_queens;
            let attacks = chess::get_rook_moves(rook, blockers);
            white_attacked |= attacks;
        }
        for queen in white_queens {
            let blockers = blockers & !white_rooks & !white_bishops & !white_queens;
            let attacks =
                chess::get_bishop_moves(queen, blockers) | chess::get_rook_moves(queen, blockers);
            white_attacked |= attacks;
        }

        for pawn in black_pawns {
            let attacks = chess::get_pawn_attacks(pawn, Color::Black, white);
            black_attacked |= attacks;
            b_pawn_attack |= attacks;
        }
        for knight in black_knights {
            let attacks = chess::get_knight_moves(knight);
            black_attacked |= attacks;
        }
        for bishop in black_bishops {
            let blockers = blockers & !black_queens & !black_bishops;
            let attacks = chess::get_bishop_moves(bishop, blockers);
            black_attacked |= attacks;
        }
        for rook in black_rooks {
            let blockers = blockers & !black_queens & !black_rooks;
            let attacks = chess::get_rook_moves(rook, blockers);
            black_attacked |= attacks;
        }
        for queen in black_queens {
            let blockers = blockers & !black_bishops & !black_rooks & !black_queens;
            let attacks =
                chess::get_bishop_moves(queen, blockers) | chess::get_rook_moves(queen, blockers);
            black_attacked |= attacks;
        }

        let w_safe_squares = !black_attacked | white_attacked;
        let w_safe_pawns = white_pawns & w_safe_squares;

        let b_safe_squares = !white_attacked | black_attacked;
        let b_safe_pawns = black_pawns & b_safe_squares;

        let white_non_pawn = white & !white_pawns;
        let black_non_pawn = black & !black_pawns;

        let mut w_pawn_threats = EMPTY;
        let mut b_pawn_threats = EMPTY;
        for pawn in w_safe_pawns {
            w_pawn_threats |= chess::get_pawn_attacks(pawn, Color::White, !EMPTY);
        }
        for pawn in b_safe_pawns {
            b_pawn_threats |= chess::get_pawn_attacks(pawn, Color::Black, !EMPTY);
        }

        let w_safe_pawn_threats = (w_pawn_threats & black_non_pawn).popcnt() as i32;
        let b_safe_pawn_threats = (b_pawn_threats & white_non_pawn).popcnt() as i32;

        let safe_pawn_threat_score = Self::score(
            w_safe_pawn_threats - b_safe_pawn_threats,
            THREAT_BY_SAFE_PAWN,
            phase,
        );

        let w_k_file = DATA.king_flank[board.king_square(Color::White).get_file() as usize];
        let b_k_file = DATA.king_flank[board.king_square(Color::Black).get_file() as usize];

        let mut flank_score = 0;
        #[cfg(feature = "new_eval")]
        {
            if w_k_file & white_pawns == EMPTY {
                flank_score -= Self::direct(PAWNLESS_FLANK, phase);
            }
            if b_k_file & black_pawns == EMPTY {
                flank_score += Self::direct(PAWNLESS_FLANK, phase);
            }
        }

        let pawn_score = self.get_pawn_score(white_pawns, black_pawns, phase);

        let white_score = psqt_score + pawn_score + safe_pawn_threat_score + flank_score;

        let score = turn * white_score + TEMPO;
        Evaluation::new(score)
    }

    fn clear_cache(&mut self) {}
}

impl BasicEval {
    //TODO: investigate tapered evaluation
    fn piece_pts(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => PAWN.0,
            Piece::Knight => KNIGHT.0,
            Piece::Bishop => BISHOP.0,
            Piece::Rook => ROOK.0,
            Piece::Queen => QUEEN.0,
            Piece::King => KING.0,
        }
    }

    fn get_pawn_score(&self, white_pawns: BitBoard, black_pawns: BitBoard, phase: i32) -> i32 {
        let mut w_passed = 0;
        let mut b_passed = 0;
        for pawn in white_pawns {
            let ahead = DATA.w_ahead[pawn.to_index()];
            w_passed += 1_u32.saturating_sub((ahead & black_pawns).popcnt());
        }
        for pawn in black_pawns {
            let ahead = DATA.b_ahead[pawn.to_index()];
            b_passed += 1_u32.saturating_sub((ahead & white_pawns).popcnt());
        }

        let mut w_doubled = 0;
        let mut b_doubled = 0;
        let mut w_isolated = 0;
        let mut b_isolated = 0;
        for &file in &ALL_FILES {
            let file_bb = chess::get_file(file);
            let adj_files = chess::get_adjacent_files(file);
            w_doubled += (file_bb & white_pawns).popcnt().saturating_sub(1);
            b_doubled += (file_bb & black_pawns).popcnt().saturating_sub(1);
            w_isolated += 1_u32.saturating_sub((adj_files & white_pawns).popcnt());
            b_isolated += 1_u32.saturating_sub((adj_files & black_pawns).popcnt());
        }
        let passed_score = Self::score(w_passed as i32 - b_passed as i32, PASSER, phase);
        let doubled_score = Self::score(w_doubled as i32 - b_doubled as i32, DOUBLED, phase);
        let isolated_score = Self::score(w_isolated as i32 - b_isolated as i32, ISOLATED, phase);

        passed_score + doubled_score + isolated_score
    }

    #[inline]
    fn get_white_psqt_score(
        board: BitBoard,
        table0: &[[i32; 8]; 8],
        table1: &[[i32; 8]; 8],
        phase: i32,
    ) -> i32 {
        let mut psqt_score = 0;
        for square in board {
            let rank = 7 - square.get_rank().to_index();
            let file = square.get_file().to_index();
            psqt_score += Self::direct(TaperedEval(table0[rank][file], table1[rank][file]), phase);
        }
        psqt_score
    }

    #[inline]
    fn get_black_psqt_score(
        board: BitBoard,
        table0: &[[i32; 8]; 8],
        table1: &[[i32; 8]; 8],
        phase: i32,
    ) -> i32 {
        let mut psqt_score = 0;
        for square in board {
            let rank = square.get_rank().to_index();
            let file = square.get_file().to_index();
            psqt_score += Self::direct(TaperedEval(table0[rank][file], table1[rank][file]), phase);
        }
        psqt_score
    }

    #[inline]
    fn score<T: EvalFactor>(score: i32, eval: T, phase: i32) -> i32 {
        eval.score(score, phase)
    }

    #[inline]
    fn direct<T: EvalFactor>(eval: T, phase: i32) -> i32 {
        eval.one(phase)
    }
}
