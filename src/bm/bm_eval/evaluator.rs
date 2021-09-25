use super::eval_access::*;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_eval::eval_consts::*;
use crate::bm::bm_util::evaluator::Evaluator;
use crate::bm::bm_util::position::Position;
use chess::{BitBoard, Board, ChessMove, Color, Piece, ALL_FILES, EMPTY};

use super::eval_access::EvalResource;

const PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalData {
    w_ahead: [BitBoard; 64],
    b_ahead: [BitBoard; 64],
    king_flank: BitBoard,
    queen_flank: BitBoard,
}

pub const fn get_basic_eval_data() -> EvalData {
    let mut data = EvalData {
        w_ahead: [BitBoard(0); 64],
        b_ahead: [BitBoard(0); 64],
        king_flank: EMPTY,
        queen_flank: EMPTY,
    };

    let mut king_flank = 0_u64;
    let mut queen_flank = 0_u64;

    let mut king_rank = 0_u8;
    while king_rank < 8 {
        let mut king_file = 0_u8;
        while king_file < 8 {
            let king = (king_rank * 8 + king_file) as usize;
            let mut w_ahead = 0_u64;
            let mut b_ahead = 0_u64;

            {
                if king_file < 4 {
                    king_flank |= 1_u64 << king;
                } else {
                    queen_flank |= 1_u64 << king;
                }

                let king_rank = king_rank as i16;
                let king_file = king_file as i16;
                let mut rank = 0_u8;
                while rank < 8 {
                    let mut file = 0_u8;
                    while file < 8 {
                        let sq = rank * 8 + file;
                        {
                            let file = file as i16;
                            let rank = rank as i16;

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
    data.king_flank = BitBoard(king_flank);
    data.queen_flank = BitBoard(queen_flank);
    data
}

const DATA: EvalData = get_basic_eval_data();

#[derive(Debug, Clone)]
pub struct StdEvaluator;

impl Evaluator for StdEvaluator {
    fn new() -> Self {
        Self
    }

    fn see(mut board: Board, mut make_move: ChessMove) -> i16 {
        let mut index = 0;
        let mut gains = [0_i16; 16];
        let target_square = make_move.get_dest();
        gains[0] = Self::piece_pts(board.piece_on(target_square).unwrap());
        'outer: for i in 1..16 {
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
            gains[i - 1] = -i16::max(-gains[i - 1], gains[i]);
        }
        gains[0]
    }

    /**
    Doesn't handle checkmates or stalemates
     */
    fn evaluate(&mut self, position: &Position) -> Evaluation {
        let board = position.board();
        let res = EvalResource::new(board);

        let turn = position.turn();

        let pawns = res.get::<Pawns>();
        let knights = res.get::<Knights>();
        let bishops = res.get::<Bishops>();
        let rooks = res.get::<Rooks>();
        let queens = res.get::<Queens>();
        let kings = res.get::<Kings>();

        let phase = TOTAL_PHASE.saturating_sub(
            pawns.popcnt() * PAWN_PHASE
                + knights.popcnt() * KNIGHT_PHASE
                + bishops.popcnt() * BISHOP_PHASE
                + rooks.popcnt() * ROOK_PHASE
                + queens.popcnt() * QUEEN_PHASE,
        ) as i16;

        let white = res.get::<White>();
        let black = res.get::<Black>();

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

        //PSQT
        let white_psqt_score = Self::get_white_psqt_score(white_pawns, &PAWN_TABLE)
            + Self::get_white_psqt_score(white_knights, &KNIGHT_TABLE)
            + Self::get_white_psqt_score(white_bishops, &BISHOP_TABLE)
            + Self::get_white_psqt_score(white_rooks, &ROOK_TABLE)
            + Self::get_white_psqt_score(white_queens, &QUEEN_TABLE)
            + Self::get_white_psqt_score(white_king, &KING_TABLE);

        let black_psqt_score = Self::get_black_psqt_score(black_pawns, &PAWN_TABLE)
            + Self::get_black_psqt_score(black_knights, &KNIGHT_TABLE)
            + Self::get_black_psqt_score(black_bishops, &BISHOP_TABLE)
            + Self::get_black_psqt_score(black_rooks, &ROOK_TABLE)
            + Self::get_black_psqt_score(black_queens, &QUEEN_TABLE)
            + Self::get_black_psqt_score(black_king, &KING_TABLE);

        let psqt_score = white_psqt_score - black_psqt_score;

        let white_attacked = res.get::<WhiteNonKingAttack>();
        let black_attacked = res.get::<BlackNonKingAttack>();

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

        let b_safe_pawn_threats = b_pawn_threats & white_non_pawn;
        let w_safe_pawn_threats = w_pawn_threats & black_non_pawn;

        let safe_pawn_threat_score = (w_safe_pawn_threats.popcnt() as i16
            - b_safe_pawn_threats.popcnt() as i16)
            * THREAT_BY_SAFE_PAWN;

        let w_king = board.king_square(Color::White);
        let b_king = board.king_square(Color::Black);

        let w_empty = if w_king.get_file().to_index() < 4 {
            DATA.king_flank
        } else {
            DATA.queen_flank
        } | pawns
            == EMPTY;
        let b_empty = if b_king.get_file().to_index() < 4 {
            DATA.king_flank
        } else {
            DATA.queen_flank
        } | pawns
            == EMPTY;

        let empty_flank_score = (w_empty as i16 - b_empty as i16) * EMPTY_FLANK;

        let pawn_score = self.get_pawn_score(white_pawns, black_pawns);

        let white_score = psqt_score + pawn_score + safe_pawn_threat_score + empty_flank_score;
        let white_score = white_score.convert(phase);
        let white_score = match Self::outcome_state(board) {
            OutcomeState::Loss => white_score - 10000,
            OutcomeState::Unknown => white_score,
            OutcomeState::Win => white_score + 10000,
            OutcomeState::Draw => white_score / 10,
            OutcomeState::LikelyLoss => {
                return Evaluation::new(white_score.min(0) * turn);
            }
            OutcomeState::LikelyWin => {
                return Evaluation::new(white_score.max(0) * turn);
            }
        };

        Evaluation::new(turn * white_score + TEMPO)
    }

    fn clear_cache(&mut self) {}
}

impl StdEvaluator {
    //TODO: investigate tapered evaluation
    fn piece_pts(piece: Piece) -> i16 {
        match piece {
            Piece::Pawn => PAWN.0,
            Piece::Knight => KNIGHT.0,
            Piece::Bishop => BISHOP.0,
            Piece::Rook => ROOK.0,
            Piece::Queen => QUEEN.0,
            Piece::King => KING.0,
        }
    }

    fn get_pawn_score(&self, white_pawns: BitBoard, black_pawns: BitBoard) -> TaperedEval {
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
        let passed_score = (w_passed as i16 - b_passed as i16) * PASSER;
        let doubled_score = (w_doubled as i16 - b_doubled as i16) * DOUBLED;
        let isolated_score = (w_isolated as i16 - b_isolated as i16) * ISOLATED;

        passed_score + doubled_score + isolated_score
    }

    #[inline]
    fn get_white_psqt_score(board: BitBoard, table: &[[TaperedEval; 8]; 8]) -> TaperedEval {
        let mut psqt_score = TaperedEval(0, 0);
        for square in board {
            let rank = 7 - square.get_rank().to_index();
            let file = square.get_file().to_index();
            psqt_score += table[rank][file];
        }
        psqt_score
    }

    #[inline]
    fn get_black_psqt_score(board: BitBoard, table: &[[TaperedEval; 8]; 8]) -> TaperedEval {
        let mut psqt_score = TaperedEval(0, 0);
        for square in board {
            let rank = square.get_rank().to_index();
            let file = square.get_file().to_index();
            psqt_score += table[rank][file]
        }
        psqt_score
    }

    fn outcome_state(board: &Board) -> OutcomeState {
        let w_checkmate = Self::can_checkmate(board, Color::White);
        let b_checkmate = Self::can_checkmate(board, Color::Black);
        assert!(!(w_checkmate == Checkmate::Certain && b_checkmate == Checkmate::Certain));
        if let Checkmate::Certain = w_checkmate {
            return OutcomeState::Win;
        } else if let Checkmate::Certain = b_checkmate {
            return OutcomeState::Loss;
        }
        match (w_checkmate, b_checkmate) {
            (Checkmate::Impossible, Checkmate::Impossible) => OutcomeState::Draw,
            (Checkmate::Impossible, Checkmate::Unknown) => OutcomeState::LikelyLoss,
            (Checkmate::Unknown, Checkmate::Impossible) => OutcomeState::LikelyWin,
            (Checkmate::Unknown, Checkmate::Unknown) => OutcomeState::Unknown,
            _ => {
                unreachable!();
            }
        }
    }

    fn can_checkmate(board: &Board, side: Color) -> Checkmate {
        let pieces = *board.color_combined(side);
        let king_only = pieces.popcnt() == 1;
        if king_only {
            return Checkmate::Impossible;
        }
        let single_piece_win =
            pieces & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen)) != EMPTY;

        let knights = *board.pieces(Piece::Knight) & pieces;
        let bishops = *board.pieces(Piece::Bishop) & pieces;

        let white_bishop = (BitBoard(WHITE_SQUARES) & bishops) != EMPTY;
        let black_bishop = (BitBoard(BLACK_SQUARES) & bishops) != EMPTY;

        let bishop_pair = white_bishop && black_bishop;
        if (bishop_pair || single_piece_win || (bishops != EMPTY && knights != EMPTY))
            && board.color_combined(!side).popcnt() == 1
        {
            return Checkmate::Certain;
        }
        let pawn_cnt = board.pieces(Piece::Pawn) & pieces;
        if pawn_cnt == EMPTY {
            Checkmate::Impossible
        } else {
            Checkmate::Unknown
        }
    }
}
