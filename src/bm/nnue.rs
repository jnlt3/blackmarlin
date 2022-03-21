use cozy_chess::{Board, Color, File, Move, Piece, Rank, Square};

use self::normal::{Dense, Incremental};

use super::bm_runner::ab_runner;

mod normal;

include!(concat!(env!("OUT_DIR"), "/nnue_weights.rs"));

const PIECES: [usize; 12] = [0, 1, 2, 3, 4, 0, 5, 6, 7, 8, 9, 0];

#[derive(Debug, Clone)]
pub struct Accumulator {
    w_input_layer: Incremental<'static, INPUT, MID>,
    b_input_layer: Incremental<'static, INPUT, MID>,
}

impl Accumulator {
    pub fn update<const INCR: bool>(
        &mut self,
        stm: Color,
        stm_king_sq: Square,
        nstm_king_sq: Square,
        sq: Square,
        piece: Piece,
        color: Color,
    ) {
        let (stm_king_sq, nstm_king_sq) = if stm == Color::White {
            (stm_king_sq as usize, nstm_king_sq as usize ^ 56)
        } else {
            (stm_king_sq as usize ^ 56, nstm_king_sq as usize)
        };

        let w_piece_index = color as usize * 5 + PIECES[piece as usize];
        let b_piece_index = (!color) as usize * 5 + PIECES[piece as usize];

        let w_index = stm_king_sq * 640 + w_piece_index * 64 + sq as usize;
        let b_index = nstm_king_sq * 640 + b_piece_index * 64 + sq as usize ^ 56;

        if INCR {
            self.w_input_layer.incr_ff::<1>(w_index);
            self.b_input_layer.incr_ff::<1>(b_index);
        } else {
            self.w_input_layer.incr_ff::<-1>(w_index);
            self.b_input_layer.incr_ff::<-1>(b_index);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Nnue {
    accumulator: Vec<Accumulator>,
    head: usize,
    out_layer: Dense<'static, { MID * 2 }, OUTPUT>,
}

impl Nnue {
    pub fn new() -> Self {
        let input_layer = Incremental::new(&INCREMENTAL, INCREMENTAL_BIAS);
        let out_layer = Dense::new(&OUT, OUT_BIAS);

        Self {
            accumulator: vec![
                Accumulator {
                    w_input_layer: input_layer.clone(),
                    b_input_layer: input_layer,
                };
                ab_runner::MAX_PLY as usize + 1
            ],
            out_layer,
            head: 0,
        }
    }

    pub fn reset(&mut self, board: &Board) {
        let stm = board.side_to_move();
        let stm_king = board.king(stm);
        let nstm_king = board.king(!stm);
        let accumulator = &mut self.accumulator[self.head];
        accumulator.w_input_layer.reset(INCREMENTAL_BIAS);
        accumulator.b_input_layer.reset(INCREMENTAL_BIAS);
        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            accumulator.update::<true>(stm, stm_king, nstm_king, sq, piece, color);
        }
    }

    pub fn full_reset(&mut self, board: &Board) {
        self.head = 0;
        self.reset(board);
    }

    pub fn null_move(&mut self) {
        self.accumulator[self.head + 1] = self.accumulator[self.head].clone();
        self.head += 1;
    }

    pub fn make_move(&mut self, board: &Board, make_move: Move) {
        self.accumulator[self.head + 1] = self.accumulator[self.head].clone();
        self.head += 1;

        let from_sq = make_move.from;
        let from_type = board.piece_on(from_sq).unwrap();
        let stm = board.side_to_move();
        let stm_king = board.king(stm);
        let nstm_king = board.king(!stm);
        if from_type == Piece::King {
            self.reset(board);
            return;
        }
        let acc = &mut self.accumulator[self.head];

        acc.update::<false>(stm, stm_king, nstm_king, from_sq, from_type, stm);

        let to_sq = make_move.to;
        if let Some((captured, color)) = board.piece_on(to_sq).zip(board.color_on(to_sq)) {
            acc.update::<false>(stm, stm_king, nstm_king, to_sq, captured, color);
        }

        if let Some(ep) = board.en_passant() {
            let (stm_fifth, stm_sixth) = match stm {
                Color::White => (Rank::Fifth, Rank::Sixth),
                Color::Black => (Rank::Fourth, Rank::Third),
            };
            if from_type == Piece::Pawn && to_sq == Square::new(ep, stm_sixth) {
                acc.update::<false>(
                    stm,
                    stm_king,
                    nstm_king,
                    Square::new(ep, stm_fifth),
                    Piece::Pawn,
                    !stm,
                );
            }
        }
        if Some(stm) == board.color_on(to_sq) {
            let stm_first = match stm {
                Color::White => Rank::First,
                Color::Black => Rank::Eighth,
            };
            if to_sq.file() > from_sq.file() {
                acc.update::<true>(
                    stm,
                    stm_king,
                    nstm_king,
                    Square::new(File::G, stm_first),
                    Piece::King,
                    stm,
                );
                acc.update::<true>(
                    stm,
                    stm_king,
                    nstm_king,
                    Square::new(File::F, stm_first),
                    Piece::Rook,
                    stm,
                );
            } else {
                acc.update::<true>(
                    stm,
                    stm_king,
                    nstm_king,
                    Square::new(File::C, stm_first),
                    Piece::King,
                    stm,
                );
                acc.update::<true>(
                    stm,
                    stm_king,
                    nstm_king,
                    Square::new(File::D, stm_first),
                    Piece::Rook,
                    stm,
                );
            }
        } else {
            acc.update::<true>(
                stm,
                stm_king,
                nstm_king,
                to_sq,
                make_move.promotion.unwrap_or(from_type),
                stm,
            );
        }
    }

    pub fn unmake_move(&mut self) {
        self.head -= 1;
    }

    #[inline]
    pub fn feed_forward(&mut self, board: &Board, bucket: usize) -> i16 {
        let acc = &mut self.accumulator[self.head];
        let mut incr = [0; MID * 2];
        let (stm, nstm) = match board.side_to_move() {
            Color::White => (&acc.w_input_layer, &acc.b_input_layer),
            Color::Black => (&acc.b_input_layer, &acc.w_input_layer),
        };
        normal::clipped_relu(*stm.get(), &mut incr);
        normal::clipped_relu(*nstm.get(), &mut incr[MID..]);

        normal::out(self.out_layer.ff(&incr, bucket)[bucket])
    }
}
