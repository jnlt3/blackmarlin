use std::sync::Arc;

use cozy_chess::{Board, Color, File, Move, Piece, Rank, Square};

use self::layers::{Dense, Incremental};

use super::bm_runner::ab_runner;

mod include;
mod layers;

include!(concat!(env!("OUT_DIR"), "/arch.rs"));

const NN_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eval.bin"));

#[derive(Debug, Clone)]
pub struct Accumulator {
    w_input_layer: Incremental<INPUT, MID>,
    b_input_layer: Incremental<INPUT, MID>,
}

fn halfka_feature(
    perspective: Color,
    king: Square,
    color: Color,
    piece: Piece,
    square: Square,
) -> usize {
    let (king, square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    let mut index = 0;
    index = index * Square::NUM + king as usize;
    index = index * Color::NUM + color as usize;
    index = index * Piece::NUM + piece as usize;
    index = index * Square::NUM + square as usize;
    index
}

impl Accumulator {
    pub fn update<const INCR: bool>(
        &mut self,
        w_king: Square,
        b_king: Square,
        sq: Square,
        piece: Piece,
        color: Color,
    ) {
        let w_index = halfka_feature(Color::White, w_king, color, piece, sq);
        let b_index = halfka_feature(Color::Black, b_king, color, piece, sq);

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
    bias: Arc<[i16; MID]>,
    head: usize,
    out_layer: Dense<{ MID * 2 }, OUTPUT>,
}

impl Nnue {
    pub fn new() -> Self {
        let mut bytes = &NN_BYTES[12..];
        let incremental = Arc::new(*include::sparse_from_bytes_i16::<i16, INPUT, MID>(bytes));
        bytes = &bytes[INPUT * MID * 2..];
        let incremental_bias = include::bias_from_bytes_i16::<i16, MID>(bytes);
        bytes = &bytes[MID * 2..];
        let out = Arc::new(*include::dense_from_bytes_i8::<i8, { MID * 2 }, OUTPUT>(
            bytes,
        ));
        bytes = &bytes[MID * OUTPUT * 2..];
        let out_bias = include::bias_from_bytes_i16::<i32, OUTPUT>(bytes);
        bytes = &bytes[OUTPUT * 2..];
        assert!(bytes.is_empty(), "{}", bytes.len());

        let input_layer = Incremental::new(incremental, incremental_bias);
        let out_layer = Dense::new(out, out_bias);

        Self {
            accumulator: vec![
                Accumulator {
                    w_input_layer: input_layer.clone(),
                    b_input_layer: input_layer,
                };
                ab_runner::MAX_PLY as usize + 1
            ],
            bias: Arc::new(incremental_bias),
            out_layer,
            head: 0,
        }
    }

    pub fn reset(&mut self, board: &Board) {
        let w_king = board.king(Color::White);
        let b_king = board.king(Color::Black);
        let acc = &mut self.accumulator[self.head];

        acc.w_input_layer.reset(*self.bias);
        acc.b_input_layer.reset(*self.bias);

        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            acc.update::<true>(w_king, b_king, sq, piece, color);
        }
    }

    pub fn full_reset(&mut self, board: &Board) {
        self.head = 0;
        self.reset(board);
    }

    fn push_accumulator(&mut self) {
        let w_out = *self.accumulator[self.head].w_input_layer.get();
        let b_out = *self.accumulator[self.head].b_input_layer.get();
        self.accumulator[self.head + 1].w_input_layer.reset(w_out);
        self.accumulator[self.head + 1].b_input_layer.reset(b_out);
        self.head += 1;
    }

    pub fn null_move(&mut self) {
        self.push_accumulator();
    }

    pub fn make_move(&mut self, board: &Board, make_move: Move) {
        self.push_accumulator();
        let from_sq = make_move.from;
        let from_type = board.piece_on(from_sq).unwrap();
        let stm = board.side_to_move();
        let w_king = board.king(Color::White);
        let b_king = board.king(Color::Black);
        if from_type == Piece::King {
            let mut board_clone = board.clone();
            board_clone.play_unchecked(make_move);
            self.reset(&board_clone);
            return;
        }
        let acc = &mut self.accumulator[self.head];

        acc.update::<false>(w_king, b_king, from_sq, from_type, stm);

        let to_sq = make_move.to;
        if let Some((captured, color)) = board.piece_on(to_sq).zip(board.color_on(to_sq)) {
            acc.update::<false>(w_king, b_king, to_sq, captured, color);
        }

        if let Some(ep) = board.en_passant() {
            let (stm_fifth, stm_sixth) = match stm {
                Color::White => (Rank::Fifth, Rank::Sixth),
                Color::Black => (Rank::Fourth, Rank::Third),
            };
            if from_type == Piece::Pawn && to_sq == Square::new(ep, stm_sixth) {
                acc.update::<false>(
                    w_king,
                    b_king,
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
                    w_king,
                    b_king,
                    Square::new(File::G, stm_first),
                    Piece::King,
                    stm,
                );
                acc.update::<true>(
                    w_king,
                    b_king,
                    Square::new(File::F, stm_first),
                    Piece::Rook,
                    stm,
                );
            } else {
                acc.update::<true>(
                    w_king,
                    b_king,
                    Square::new(File::C, stm_first),
                    Piece::King,
                    stm,
                );
                acc.update::<true>(
                    w_king,
                    b_king,
                    Square::new(File::D, stm_first),
                    Piece::Rook,
                    stm,
                );
            }
        } else {
            acc.update::<true>(
                w_king,
                b_king,
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
    pub fn feed_forward(&mut self, stm: Color) -> i16 {
        let acc = &mut self.accumulator[self.head];
        let mut incr = [0; MID * 2];
        let (stm, nstm) = match stm {
            Color::White => (&acc.w_input_layer, &acc.b_input_layer),
            Color::Black => (&acc.b_input_layer, &acc.w_input_layer),
        };
        layers::sq_clipped_relu(*stm.get(), &mut incr);
        layers::sq_clipped_relu(*nstm.get(), &mut incr[MID..]);

        layers::out(self.out_layer.ff(&incr)[0])
    }
}
