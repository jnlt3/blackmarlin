use crate::bm::bm_eval::basic_eval_consts::*;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_util::evaluator::Evaluator;
use crate::bm::bm_util::position::Position;
use chess::{BitBoard, Board, ChessMove, Color, Piece, ALL_FILES, EMPTY};

use std::sync::Arc;

const PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

#[derive(Debug, Clone)]
pub struct BasicEval {
    w_king_attacks: Arc<[BitBoard; 64]>,
    b_king_attacks: Arc<[BitBoard; 64]>,
    w_ahead: Arc<[BitBoard; 64]>,
    b_ahead: Arc<[BitBoard; 64]>,
}

impl Evaluator for BasicEval {
    fn new() -> Self {
        let mut w_king_attacks = [EMPTY; 64];
        let mut b_king_attacks = [EMPTY; 64];
        let mut w_ahead = [EMPTY; 64];
        let mut b_ahead = [EMPTY; 64];
        for king in !EMPTY {
            let king_file = king.get_file().to_index() as i32;
            let king_rank = king.get_rank().to_index() as i32;
            let w_king_bb = &mut w_king_attacks[king.to_index()];
            let b_king_bb = &mut b_king_attacks[king.to_index()];

            let w_ahead_bb = &mut w_ahead[king.to_index()];
            let b_ahead_bb = &mut b_ahead[king.to_index()];

            for sq in !EMPTY {
                let file = sq.get_file().to_index() as i32;
                let rank = sq.get_rank().to_index() as i32;
                let rank_diff = rank - king_rank;
                if (file - king_file).abs() <= 1 && rank_diff >= -1 && rank_diff <= 2 {
                    *w_king_bb |= BitBoard::from_square(sq);
                }
                if (file - king_file).abs() <= 1 && rank_diff > 0 {
                    *w_ahead_bb |= BitBoard::from_square(sq);
                }
                let rank_diff = king_rank - rank;
                if (file - king_file).abs() <= 1 && rank_diff >= -1 && rank_diff <= 2 {
                    *b_king_bb |= BitBoard::from_square(sq);
                }
                if (file - king_file).abs() <= 1 && rank_diff > 0 {
                    *b_ahead_bb |= BitBoard::from_square(sq);
                }
            }
        }
        Self {
            w_king_attacks: Arc::new(w_king_attacks),
            b_king_attacks: Arc::new(b_king_attacks),
            w_ahead: Arc::new(w_ahead),
            b_ahead: Arc::new(b_ahead),
        }
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
        let mut phase = TOTAL_PHASE;

        let pawns = *board.pieces(Piece::Pawn);
        let knights = *board.pieces(Piece::Knight);
        let bishops = *board.pieces(Piece::Bishop);
        let rooks = *board.pieces(Piece::Rook);
        let queens = *board.pieces(Piece::Queen);
        let kings = *board.pieces(Piece::King);

        phase = phase.saturating_sub(pawns.popcnt() * PAWN_PHASE);
        phase = phase.saturating_sub(knights.popcnt() * KNIGHT_PHASE);
        phase = phase.saturating_sub(bishops.popcnt() * BISHOP_PHASE);
        phase = phase.saturating_sub(rooks.popcnt() * ROOK_PHASE);
        phase = phase.saturating_sub(queens.popcnt() * QUEEN_PHASE);
        let phase = phase as i32;

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
        let pawn_phased_score = Self::direct(PAWN, phase);
        let knight_phased_score = Self::direct(KNIGHT, phase);
        let bishop_phased_score = Self::direct(BISHOP, phase);
        let rook_phased_score = Self::direct(ROOK, phase);
        let queen_phased_score = Self::direct(QUEEN, phase);

        //Material
        let white_material = white_pawns.popcnt() as i32 * pawn_phased_score
            + white_knights.popcnt() as i32 * knight_phased_score
            + white_bishops.popcnt() as i32 * bishop_phased_score
            + white_rooks.popcnt() as i32 * rook_phased_score
            + white_queens.popcnt() as i32 * queen_phased_score;

        let black_material = black_pawns.popcnt() as i32 * pawn_phased_score
            + black_knights.popcnt() as i32 * knight_phased_score
            + black_bishops.popcnt() as i32 * bishop_phased_score
            + black_rooks.popcnt() as i32 * rook_phased_score
            + black_queens.popcnt() as i32 * queen_phased_score;

        let material_score = white_material - black_material;

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
        let w_king = board.king_square(Color::White);
        let b_king = board.king_square(Color::Black);

        let mut w_attack_cnt = 0_usize;
        let mut b_attack_cnt = 0_usize;

        let mut w_attack = 0;
        let mut b_attack = 0;

        for knight in white_knights {
            let attacks = chess::get_knight_moves(knight);
            let table = self.b_king_attacks[b_king.to_index()];
            let king_attacks = attacks & table;
            w_attack_cnt += (king_attacks != EMPTY) as usize;
            w_attack += Self::score(king_attacks.popcnt() as i32, KNIGHT_ATTACK, phase);
        }
        for bishop in white_bishops {
            let blockers = blockers & !white_bishops & !white_queens;
            let attacks = chess::get_bishop_moves(bishop, blockers);
            let table = self.b_king_attacks[b_king.to_index()];
            let king_attacks = attacks & table;
            w_attack_cnt += (king_attacks != EMPTY) as usize;
            w_attack += Self::score(king_attacks.popcnt() as i32, BISHOP_ATTACK, phase);
        }
        for rook in white_rooks {
            let blockers = blockers & !white_rooks & !white_queens;
            let attacks = chess::get_rook_moves(rook, blockers);
            let table = self.b_king_attacks[b_king.to_index()];
            let king_attacks = attacks & table;
            w_attack_cnt += (king_attacks != EMPTY) as usize;
            w_attack += Self::score(king_attacks.popcnt() as i32, ROOK_ATTACK, phase);
        }
        for queen in white_queens {
            let blockers = blockers & !white_rooks & !white_bishops & !white_queens;
            let attacks =
                chess::get_bishop_moves(queen, blockers) | chess::get_rook_moves(queen, blockers);
            let table = self.b_king_attacks[b_king.to_index()];
            let king_attacks = attacks & table;
            w_attack_cnt += (king_attacks != EMPTY) as usize;
            w_attack += Self::score(king_attacks.popcnt() as i32, QUEEN_ATTACK, phase);
        }
        for knight in black_knights {
            let attacks = chess::get_knight_moves(knight);
            let table = self.w_king_attacks[w_king.to_index()];
            let king_attacks = attacks & table;
            b_attack_cnt += (king_attacks != EMPTY) as usize;
            b_attack += Self::score(king_attacks.popcnt() as i32, KNIGHT_ATTACK, phase);
        }
        for bishop in black_bishops {
            let blockers = blockers & !black_queens & !black_bishops;
            let attacks = chess::get_bishop_moves(bishop, blockers);
            let table = self.w_king_attacks[w_king.to_index()];
            let king_attacks = attacks & table;
            b_attack_cnt += (king_attacks != EMPTY) as usize;
            b_attack += Self::score(king_attacks.popcnt() as i32, BISHOP_ATTACK, phase);
        }
        for rook in black_rooks {
            let blockers = blockers & !black_queens & !black_rooks;
            let attacks = chess::get_rook_moves(rook, blockers);
            let table = self.w_king_attacks[w_king.to_index()];
            let king_attacks = attacks & table;
            b_attack_cnt += (king_attacks != EMPTY) as usize;
            b_attack += Self::score(king_attacks.popcnt() as i32, ROOK_ATTACK, phase);
        }
        for queen in black_queens {
            let blockers = blockers & !black_bishops & !black_rooks & !black_queens;
            let attacks =
                chess::get_bishop_moves(queen, blockers) | chess::get_rook_moves(queen, blockers);
            let table = self.w_king_attacks[w_king.to_index()];
            let king_attacks = attacks & table;
            b_attack_cnt += (king_attacks != EMPTY) as usize;
            b_attack += Self::score(king_attacks.popcnt() as i32, QUEEN_ATTACK, phase);
        }

        let w_attacker_count = w_attack_cnt.min(ATTACKS.len() - 1);
        let b_attacker_count = b_attack_cnt.min(ATTACKS.len() - 1);

        let w_attack_score = w_attack as i32 * ATTACKS[w_attacker_count];
        let b_attack_score = b_attack as i32 * ATTACKS[b_attacker_count];
        let attack_score = (w_attack_score - b_attack_score) / 100;

        let pawn_score = self.get_pawn_score(white_pawns, black_pawns, phase);

        let score = turn * (material_score + psqt_score + attack_score + pawn_score) + TEMPO;
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
            let ahead = self.w_ahead[pawn.to_index()];
            w_passed += 1u32.saturating_sub((ahead & black_pawns).popcnt());
        }
        for pawn in black_pawns {
            let ahead = self.b_ahead[pawn.to_index()];
            b_passed += 1u32.saturating_sub((ahead & white_pawns).popcnt());
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
            w_isolated += 1u32.saturating_sub((adj_files & white_pawns).popcnt());
            b_isolated += 1u32.saturating_sub((adj_files & black_pawns).popcnt());
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
            psqt_score += Self::score(
                1,
                TaperedEval(table0[rank][file], table1[rank][file]),
                phase,
            );
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
