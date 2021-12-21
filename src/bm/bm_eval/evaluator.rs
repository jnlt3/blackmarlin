use crate::bm::bm_eval::eval::Evaluation;
#[cfg(not(feature = "nnue"))]
use crate::bm::bm_eval::eval_consts::*;
#[cfg(feature = "nnue")]
use crate::bm::nnue::Nnue;
#[cfg(feature = "trace")]
use arrayvec::ArrayVec;
#[cfg(not(feature = "nnue"))]
use chess::ALL_FILES;
use chess::{BitBoard, Board, ChessMove, Color, Piece, ALL_PIECES, EMPTY};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalData {
    pub w_ahead: [BitBoard; 64],
    pub b_ahead: [BitBoard; 64],
    pub w_protector: [BitBoard; 64],
    pub b_protector: [BitBoard; 64],
    pub ring: [BitBoard; 64],
}

#[cfg(not(feature = "nnue"))]
pub const fn get_basic_eval_data() -> EvalData {
    let mut data = EvalData {
        w_ahead: [BitBoard(0); 64],
        b_ahead: [BitBoard(0); 64],
        w_protector: [BitBoard(0); 64],
        b_protector: [BitBoard(0); 64],
        ring: [BitBoard(0); 64],
    };

    let mut king_rank = 0_u8;
    while king_rank < 8 {
        let mut king_file = 0_u8;
        while king_file < 8 {
            let king = (king_rank * 8 + king_file) as usize;
            let mut w_ahead = 0_u64;
            let mut b_ahead = 0_u64;

            let mut w_protector = 0_u64;
            let mut b_protector = 0_u64;

            let mut ring = 0_u64;

            {
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
                            if file_diff <= 1 && rank_diff > -1 {
                                w_protector |= bitboard;
                                if rank_diff > 0 {
                                    w_ahead |= bitboard;
                                }
                            }
                            let rank_diff = king_rank - rank;
                            if file_diff <= 1 && rank_diff > -1 {
                                b_protector |= bitboard;
                                if rank_diff > 0 {
                                    b_ahead |= bitboard
                                }
                            }
                            if file_diff <= 2 && rank_diff.abs() <= 2 {
                                ring |= bitboard;
                            }
                        }
                        file += 1;
                    }
                    rank += 1;
                }
            }
            data.w_ahead[king] = BitBoard(w_ahead);
            data.b_ahead[king] = BitBoard(b_ahead);
            data.w_protector[king] = BitBoard(w_protector);
            data.b_protector[king] = BitBoard(b_protector);
            data.ring[king] = BitBoard(ring);
            king_file += 1;
        }
        king_rank += 1;
    }
    data
}

#[cfg(not(feature = "nnue"))]
pub const DATA: EvalData = get_basic_eval_data();

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PsqtTrace {
    pub bb: BitBoard,
}
#[cfg(feature = "trace")]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EvalTrace {
    pub phase: i16,
    pub tempo: i16,
    pub passed: i16,
    pub doubled: i16,
    pub isolated: i16,
    pub chained: i16,
    pub phalanx: i16,
    pub threat: i16,
    pub passed_table: RanksPair,

    pub bishop_pair: i16,

    pub knight_attack_cnt: i16,
    pub bishop_attack_cnt: i16,
    pub rook_attack_cnt: i16,
    pub queen_attack_cnt: i16,

    pub attackers: Indices<1, 16>,

    pub knight_mobility: Indices<6, 9>,
    pub bishop_mobility: Indices<6, 14>,
    pub rook_mobility: Indices<6, 15>,
    pub queen_mobility: Indices<6, 28>,

    pub pawn_cnt: i16,
    pub knight_cnt: i16,
    pub bishop_cnt: i16,
    pub rook_cnt: i16,
    pub queen_cnt: i16,
    pub king_cnt: i16,

    pub pawns: BbPair,
    pub knights: BbPair,
    pub bishops: BbPair,
    pub rooks: BbPair,
    pub queens: BbPair,
    pub kings: BbPair,
}

#[cfg(feature = "trace")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct BbPair(pub BitBoard, pub BitBoard);

#[cfg(feature = "trace")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct RanksPair(pub BitBoard, pub BitBoard);

#[cfg(feature = "trace")]
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Indices<const CAP: usize, const SIZE: usize>(
    pub ArrayVec<u8, CAP>,
    pub ArrayVec<u8, CAP>,
);

#[cfg(not(feature = "nnue"))]
macro_rules! reset_trace {
    ($trace: expr) => {
        #[cfg(feature = "trace")]
        {
            let trace: &mut EvalTrace = $trace;
            *trace = Default::default();
        }
    };
}

#[cfg(not(feature = "nnue"))]
macro_rules! trace_tempo {
    ($trace: expr, $color: expr) => {
        #[cfg(feature = "trace")]
        {
            $trace.tempo = match $color {
                Color::White => 1,
                Color::Black => -1,
            }
        }
    };
}

#[cfg(not(feature = "nnue"))]
macro_rules! trace_phase {
    ($trace: expr, $phase: expr) => {
        #[cfg(feature = "trace")]
        {
            $trace.phase = $phase;
        }
    };
}

#[cfg(not(feature = "nnue"))]
macro_rules! trace_eval {
    ($trace: expr, $field: ident) => {
        #[cfg(feature = "trace")]
        {
            $trace.$field = $field;
        }
    };
    ($trace: expr, $field: ident, $($fields: ident),*) => {
        #[cfg(feature = "trace")]
        {
            $trace.$field = $field;
            trace_eval!($trace, $($fields),*);
        }
    }
}

#[cfg(not(feature = "nnue"))]
macro_rules! trace_index {
    ($trace: expr, $field: ident, $color: expr, $index: expr) => {
        #[cfg(feature = "trace")]
        {
            let index: u8 = $index as u8;
            let color: Color = $color;
            match color {
                Color::White => $trace.$field.0.push(index),
                Color::Black => $trace.$field.1.push(index),
            };
        }
    };
}

#[cfg(not(feature = "nnue"))]
macro_rules! trace_ranks_pair {
    ($trace: expr, $field: ident, $bb_0: expr, $bb_1: expr) => {
        #[cfg(feature = "trace")]
        {
            let trace: &mut EvalTrace = $trace;
            let bb_0: BitBoard = $bb_0;
            let bb_1: BitBoard = $bb_1;
            trace.$field = RanksPair(bb_0, bb_1.reverse_colors());
        }
    };
}

#[cfg(not(feature = "nnue"))]
macro_rules! trace_psqt {
    ($trace: expr, $piece: ident, $piece_cnt: ident, $bitboard_0: expr, $bitboard_1: expr) => {
        #[cfg(feature = "trace")]
        {
            let trace: &mut EvalTrace = $trace;
            let bitboard_0: BitBoard = $bitboard_0;
            let bitboard_1: BitBoard = $bitboard_1;
            trace.$piece = BbPair(bitboard_0, bitboard_1.reverse_colors());
            trace.$piece_cnt = bitboard_0.popcnt() as i16 - bitboard_1.popcnt() as i16;
        }
    };
}

#[derive(Debug, Clone)]
pub struct StdEvaluator {
    #[cfg(feature = "trace")]
    trace: EvalTrace,

    #[cfg(feature = "nnue")]
    nnue: Nnue,
}

#[cfg(feature = "nnue")]
const NNUE_TEMPO: i16 = 15;

impl StdEvaluator {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "trace")]
            trace: Default::default(),
            #[cfg(feature = "nnue")]
            nnue: Nnue::new(),
        }
    }

    pub fn see(mut board: Board, mut make_move: ChessMove) -> i16 {
        let mut index = 0;
        let mut gains = [0_i16; 16];
        let target_square = make_move.get_dest();
        gains[0] = if let Some(piece) = board.piece_on(target_square) {
            Self::piece_pts(piece)
        } else {
            0
        };
        'outer: for i in 1..16 {
            board = board.make_move_new(make_move);
            gains[i] = Self::piece_pts(board.piece_on(target_square).unwrap()) - gains[i - 1];
            let color = board.side_to_move();
            let defenders = *board.color_combined(color);
            let blockers = *board.combined();
            for piece in &ALL_PIECES {
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

    pub fn insufficient_material(&self, board: &Board) -> bool {
        if board.combined().popcnt() == 2 {
            true
        } else if board.combined().popcnt() == 3 {
            (board.pieces(Piece::Rook) | board.pieces(Piece::Queen) | board.pieces(Piece::Pawn))
                == EMPTY
        } else {
            false
        }
    }

    //TODO: Later to be removed with new NNUE versions
    #[cfg(feature = "nnue")]
    fn eval_scale(board: &Board) -> f32 {
        let pawns = *board.pieces(Piece::Pawn);
        let pieces = board.combined() & !pawns;
        let queens = *board.pieces(Piece::Queen);
        let pawn_cnt = pawns.popcnt();
        let piece_cnt = pieces.popcnt();
        let queen_cnt = queens.popcnt();
        pawn_cnt as f32 * (1.0 / 24.0)
            + piece_cnt as f32 * (1.0 / 16.0)
            + queen_cnt as f32 * (1.0 / 8.0)
    }

    /**
    Doesn't handle checkmates or stalemates
     */
    pub fn evaluate(&mut self, board: &Board) -> Evaluation {
        if self.insufficient_material(board) {
            return Evaluation::new(0);
        }
        let turn = match board.side_to_move() {
            Color::White => 1,
            Color::Black => -1,
        };
        #[cfg(feature = "nnue")]
        {
            let scale = Self::eval_scale(board);
            let nnue_out = self.nnue.feed_forward(board, 0);
            let scaled = (nnue_out as f32 * scale) as i16;
            return Evaluation::new(scaled * turn + NNUE_TEMPO);
        }
        #[cfg(not(feature = "nnue"))]
        {
            let phase = (board.pieces(Piece::Pawn).popcnt() * PAWN_PHASE
                + board.pieces(Piece::Knight).popcnt() * KNIGHT_PHASE
                + board.pieces(Piece::Bishop).popcnt() * BISHOP_PHASE
                + board.pieces(Piece::Rook).popcnt() * ROOK_PHASE
                + board.pieces(Piece::Queen).popcnt() * QUEEN_PHASE)
                .min(TOTAL_PHASE) as i16;
            reset_trace!(&mut self.trace);
            trace_tempo!(&mut self.trace, board.side_to_move());

            trace_phase!(&mut self.trace, phase);

            let eval = self.evaluate_psqt(board, Piece::Pawn)
                + self.evaluate_psqt(board, Piece::Knight)
                + self.evaluate_psqt(board, Piece::Bishop)
                + self.evaluate_psqt(board, Piece::Rook)
                + self.evaluate_psqt(board, Piece::Queen)
                + self.evaluate_psqt(board, Piece::King)
                + self.evaluate_pawns(board)
                + self.evaluate_bishops(board)
                + self.evaluate_threats(board);

            Evaluation::new((eval * turn + TEMPO).convert(phase))
        }
    }

    //TODO: investigate tapered evaluation
    fn piece_pts(piece: Piece) -> i16 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 300,
            Piece::Bishop => 300,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
    }

    #[cfg(feature = "trace")]
    pub fn get_trace(&self) -> &EvalTrace {
        &self.trace
    }

    #[cfg(not(feature = "nnue"))]
    fn evaluate_threats(&mut self, board: &Board) -> TaperedEval {
        let blockers = *board.combined();

        let whites = *board.color_combined(Color::White);
        let blacks = *board.color_combined(Color::Black);

        let pawns = *board.pieces(Piece::Pawn);
        let knights = *board.pieces(Piece::Knight);
        let bishops = *board.pieces(Piece::Bishop);
        let rooks = *board.pieces(Piece::Rook);
        let queens = *board.pieces(Piece::Queen);
        let kings = *board.pieces(Piece::King);

        let w_king = board.king_square(Color::White);
        let b_king = board.king_square(Color::Black);

        let w_king_ring = DATA.ring[w_king.to_index()];
        let b_king_ring = DATA.ring[b_king.to_index()];

        let w_knight_attack = chess::get_knight_moves(b_king);
        let b_knight_attack = chess::get_knight_moves(w_king);

        let w_bishop_attack = chess::get_bishop_moves(b_king, blockers);
        let b_bishop_attack = chess::get_bishop_moves(w_king, blockers);

        let w_rook_attack = chess::get_rook_moves(b_king, blockers);
        let b_rook_attack = chess::get_rook_moves(w_king, blockers);

        let w_queen_attack =
            chess::get_bishop_moves(b_king, blockers) | chess::get_rook_moves(b_king, blockers);
        let b_queen_attack =
            chess::get_bishop_moves(w_king, blockers) | chess::get_rook_moves(w_king, blockers);

        let mut knight_attack_cnt = 0_i16;
        let mut bishop_attack_cnt = 0_i16;
        let mut rook_attack_cnt = 0_i16;
        let mut queen_attack_cnt = 0_i16;

        let mut w_pawn_attacks = EMPTY;
        let mut b_pawn_attacks = EMPTY;

        for pawn in pawns & whites {
            w_pawn_attacks |= chess::get_pawn_attacks(pawn, Color::White, !EMPTY);
        }
        for pawn in pawns & blacks {
            b_pawn_attacks |= chess::get_pawn_attacks(pawn, Color::Black, !EMPTY);
        }

        let w_mobility_area = !(b_pawn_attacks | (kings & whites) | (whites & pawns));
        let b_mobility_area = !(w_pawn_attacks | (kings & blacks) | (blacks & pawns));

        let mut knight_mobility = TaperedEval(0, 0);
        let mut bishop_mobility = TaperedEval(0, 0);
        let mut rook_mobility = TaperedEval(0, 0);
        let mut queen_mobility = TaperedEval(0, 0);

        let mut white_attackers = 0_usize;
        let mut black_attackers = 0_usize;

        for knight in knights & whites {
            let attacks = chess::get_knight_moves(knight);
            let mobility = (attacks & w_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, knight_mobility, Color::White, mobility);
            knight_mobility += KNIGHT_MOBILITY[mobility];
            if w_knight_attack & attacks != EMPTY {
                knight_attack_cnt += 1;
            }
            if b_king_ring & attacks != EMPTY {
                white_attackers += 1;
            }
        }
        for knight in knights & blacks {
            let attacks = chess::get_knight_moves(knight);
            let mobility = (attacks & b_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, knight_mobility, Color::Black, mobility);
            knight_mobility -= KNIGHT_MOBILITY[mobility];
            if b_knight_attack & attacks != EMPTY {
                knight_attack_cnt -= 1;
            }
            if w_king_ring & attacks != EMPTY {
                black_attackers += 1;
            }
        }

        for diag in bishops & whites {
            let attacks = chess::get_bishop_moves(diag, blockers);
            let mobility = (attacks & w_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, bishop_mobility, Color::White, mobility);
            bishop_mobility += BISHOP_MOBILITY[mobility];
            if w_bishop_attack & attacks != EMPTY {
                bishop_attack_cnt += 1;
            }
            if b_king_ring & attacks != EMPTY {
                white_attackers += 1;
            }
        }
        for diag in bishops & blacks {
            let attacks = chess::get_bishop_moves(diag, blockers);
            let mobility = (attacks & b_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, bishop_mobility, Color::Black, mobility);
            bishop_mobility -= BISHOP_MOBILITY[mobility];
            if b_bishop_attack & attacks != EMPTY {
                bishop_attack_cnt -= 1;
            }
            if w_king_ring & attacks != EMPTY {
                black_attackers += 1;
            }
        }

        for ortho in rooks & whites {
            let attacks = chess::get_rook_moves(ortho, blockers);
            let mobility = (attacks & w_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, rook_mobility, Color::White, mobility);
            rook_mobility += ROOK_MOBILITY[mobility];
            if w_rook_attack & attacks != EMPTY {
                rook_attack_cnt += 1;
            }
            if b_king_ring & attacks != EMPTY {
                white_attackers += 1;
            }
        }
        for ortho in rooks & blacks {
            let attacks = chess::get_rook_moves(ortho, blockers);
            let mobility = (attacks & b_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, rook_mobility, Color::Black, mobility);
            rook_mobility -= ROOK_MOBILITY[mobility];
            if b_rook_attack & attacks != EMPTY {
                rook_attack_cnt -= 1;
            }
            if w_king_ring & attacks != EMPTY {
                black_attackers += 1;
            }
        }

        for queen in queens & whites {
            let attacks =
                chess::get_bishop_moves(queen, blockers) | chess::get_rook_moves(queen, blockers);
            let mobility = (attacks & w_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, queen_mobility, Color::White, mobility);
            queen_mobility += QUEEN_MOBILITY[mobility];
            if w_queen_attack & attacks != EMPTY {
                queen_attack_cnt += 1;
            }
            if b_king_ring & attacks != EMPTY {
                white_attackers += 1;
            }
        }
        for queen in queens & blacks {
            let attacks =
                chess::get_bishop_moves(queen, blockers) | chess::get_rook_moves(queen, blockers);
            let mobility = (attacks & b_mobility_area).popcnt() as usize;
            trace_index!(&mut self.trace, queen_mobility, Color::Black, mobility);
            queen_mobility -= QUEEN_MOBILITY[mobility];
            if b_queen_attack & attacks != EMPTY {
                queen_attack_cnt -= 1;
            }
            if w_king_ring & attacks != EMPTY {
                black_attackers += 1;
            }
        }

        let white_in_king_ring = (whites & b_king_ring & !pawns).popcnt().min(3) as usize;
        let black_in_king_ring = (blacks & w_king_ring & !pawns).popcnt().min(3) as usize;
        let white_attackers = white_attackers.min(3);
        let black_attackers = black_attackers.min(3);

        let w_index = white_attackers * 4 + white_in_king_ring;
        let b_index = black_attackers * 4 + black_in_king_ring;

        trace_index!(&mut self.trace, attackers, Color::White, w_index as u8);
        trace_index!(&mut self.trace, attackers, Color::Black, b_index as u8);

        trace_eval!(
            &mut self.trace,
            knight_attack_cnt,
            bishop_attack_cnt,
            rook_attack_cnt,
            queen_attack_cnt
        );

        let attackers = ATTACKERS[w_index] - ATTACKERS[b_index];

        knight_attack_cnt * KNIGHT_ATTACK_CNT
            + bishop_attack_cnt * BISHOP_ATTACK_CNT
            + rook_attack_cnt * ROOK_ATTACK_CNT
            + queen_attack_cnt * QUEEN_ATTACK_CNT
            + knight_mobility
            + bishop_mobility
            + rook_mobility
            + queen_mobility
            + attackers
    }

    #[cfg(not(feature = "nnue"))]
    fn evaluate_bishops(&mut self, board: &Board) -> TaperedEval {
        let bishops = *board.pieces(Piece::Bishop);
        let w_bishops = bishops & *board.color_combined(Color::White);
        let b_bishops = bishops & *board.color_combined(Color::Black);
        let w_pair = if w_bishops.popcnt() > 1 { 1 } else { 0 };
        let b_pair = if b_bishops.popcnt() > 1 { 1 } else { 0 };
        let bishop_pair = w_pair - b_pair;
        trace_eval!(&mut self.trace, bishop_pair);
        bishop_pair * BISHOP_PAIR
    }

    #[cfg(not(feature = "nnue"))]
    fn evaluate_pawns(&mut self, board: &Board) -> TaperedEval {
        let white_pawns = *board.pieces(Piece::Pawn) & board.color_combined(Color::White);
        let black_pawns = *board.pieces(Piece::Pawn) & board.color_combined(Color::Black);

        let mut w_passed_bb = EMPTY;
        let mut b_passed_bb = EMPTY;

        let mut w_isolated = 0_i16;
        let mut b_isolated = 0_i16;

        let white_non_pawn = board.color_combined(Color::White) & !white_pawns;
        let black_non_pawn = board.color_combined(Color::Black) & !black_pawns;

        let mut w_pawn_attacks = EMPTY;
        let mut b_pawn_attacks = EMPTY;

        for pawn in white_pawns {
            let ahead = DATA.w_ahead[pawn.to_index()];

            if (ahead & black_pawns) == EMPTY {
                w_passed_bb |= BitBoard::from_square(pawn);
            }

            let adj = chess::get_adjacent_files(pawn.get_file());
            if adj & white_pawns == EMPTY {
                w_isolated += 1;
            }

            let attacks = chess::get_pawn_attacks(pawn, Color::White, !EMPTY);
            w_pawn_attacks |= attacks;
        }

        for pawn in black_pawns {
            let ahead = DATA.b_ahead[pawn.to_index()];

            if (ahead & white_pawns) == EMPTY {
                b_passed_bb |= BitBoard::from_square(pawn);
            }

            let adj = chess::get_adjacent_files(pawn.get_file());
            if adj & black_pawns == EMPTY {
                b_isolated += 1;
            }

            let attacks = chess::get_pawn_attacks(pawn, Color::Black, !EMPTY);
            b_pawn_attacks |= attacks;
        }
        let mut w_doubled = 0;
        let mut b_doubled = 0;
        for &file in &ALL_FILES {
            let file_bb = chess::get_file(file);
            w_doubled += (file_bb & white_pawns).popcnt().saturating_sub(1);
            b_doubled += (file_bb & black_pawns).popcnt().saturating_sub(1);
        }

        let mut passer_score = TaperedEval(0, 0);
        for sq in w_passed_bb {
            let rank = sq.get_rank();
            passer_score += PASSED_TABLE[rank.to_index()];
        }
        for sq in b_passed_bb.reverse_colors() {
            let rank = sq.get_rank();
            passer_score -= PASSED_TABLE[rank.to_index()];
        }

        let w_phalanx = (white_pawns & BitBoard(white_pawns.0 << 1)).popcnt();
        let b_phalanx = (black_pawns & BitBoard(black_pawns.0 << 1)).popcnt();

        let w_attacks = (w_pawn_attacks & black_non_pawn).popcnt();
        let b_attacks = (b_pawn_attacks & white_non_pawn).popcnt();

        let w_chained = (w_pawn_attacks & white_pawns).popcnt();
        let b_chained = (b_pawn_attacks & black_pawns).popcnt();

        let isolated = w_isolated - b_isolated;
        let doubled = w_doubled as i16 - b_doubled as i16;
        let threat = w_attacks as i16 - b_attacks as i16;
        let chained = w_chained as i16 - b_chained as i16;
        let phalanx = w_phalanx as i16 - b_phalanx as i16;

        trace_eval!(&mut self.trace, isolated, doubled, threat, chained, phalanx);

        trace_ranks_pair!(&mut self.trace, passed_table, w_passed_bb, b_passed_bb);

        passer_score
            + isolated * ISOLATED
            + doubled * DOUBLED
            + threat * THREAT
            + chained * CHAINED
            + phalanx * PHALANX
    }

    #[cfg(not(feature = "nnue"))]
    #[inline]
    pub fn get_psqt_score(board: BitBoard, table: &[[TaperedEval; 8]; 8]) -> TaperedEval {
        let mut psqt_score = TaperedEval(0, 0);
        for square in board {
            let rank = square.get_rank().to_index();
            let file = square.get_file().to_index();
            psqt_score += table[rank][file];
        }
        psqt_score
    }

    #[cfg(not(feature = "nnue"))]
    fn evaluate_psqt(&mut self, board: &Board, piece: Piece) -> TaperedEval {
        let pieces_white = board.pieces(piece) & board.color_combined(Color::White);
        let pieces_black = board.pieces(piece) & board.color_combined(Color::Black);

        let psqt = match piece {
            Piece::Pawn => {
                trace_psqt!(&mut self.trace, pawns, pawn_cnt, pieces_white, pieces_black);
                &PAWN_TABLE
            }
            Piece::Knight => {
                trace_psqt!(
                    &mut self.trace,
                    knights,
                    knight_cnt,
                    pieces_white,
                    pieces_black
                );
                &KNIGHT_TABLE
            }
            Piece::Bishop => {
                trace_psqt!(
                    &mut self.trace,
                    bishops,
                    bishop_cnt,
                    pieces_white,
                    pieces_black
                );
                &BISHOP_TABLE
            }
            Piece::Rook => {
                trace_psqt!(&mut self.trace, rooks, rook_cnt, pieces_white, pieces_black);
                &ROOK_TABLE
            }
            Piece::Queen => {
                trace_psqt!(
                    &mut self.trace,
                    queens,
                    queen_cnt,
                    pieces_white,
                    pieces_black
                );
                &QUEEN_TABLE
            }
            Piece::King => {
                trace_psqt!(&mut self.trace, kings, king_cnt, pieces_white, pieces_black);
                &KING_TABLE
            }
        };

        Self::get_psqt_score(pieces_white, psqt)
            - Self::get_psqt_score(pieces_black.reverse_colors(), psqt)
    }
}
