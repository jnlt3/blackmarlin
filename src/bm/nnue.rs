use cozy_chess::{Board, Color, File, Move, Piece, Rank, Square};

use self::normal::{Dense, Incremental};

use super::bm_runner::ab_runner;

mod normal;

include!(concat!(env!("OUT_DIR"), "/nnue_weights.rs"));

#[derive(Debug, Clone)]
pub struct Accumulator {
    w_input_layer: Incremental<'static, INPUT, MID>,
    b_input_layer: Incremental<'static, INPUT, MID>,
}

impl Accumulator {
    pub fn update<const INCR: bool>(&mut self, sq: Square, piece: Piece, color: Color) {
        let w_piece_index = color as usize * 6 + piece as usize;
        let b_piece_index = (!color) as usize * 6 + piece as usize;

        let w_index = sq as usize + w_piece_index * 64;
        let b_index = (sq as usize ^ 56) + b_piece_index * 64;

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
        self.head = 0;
        let accumulator = &mut self.accumulator[0];
        accumulator.w_input_layer.reset(INCREMENTAL_BIAS);
        accumulator.b_input_layer.reset(INCREMENTAL_BIAS);
        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            accumulator.update::<true>(sq, piece, color);
        }
    }

    pub fn null_move(&mut self) {
        self.accumulator[self.head + 1] = self.accumulator[self.head].clone();
        self.head += 1;
    }

    pub fn make_move(&mut self, board: &Board, make_move: Move) {
        self.accumulator[self.head + 1] = self.accumulator[self.head].clone();
        self.head += 1;
        let acc = &mut self.accumulator[self.head];

        let from_sq = make_move.from;
        let from_type = board.piece_on(from_sq).unwrap();
        let from_color = board.side_to_move();
        acc.update::<false>(from_sq, from_type, from_color);

        let to_sq = make_move.to;
        if let Some((captured, color)) = board.piece_on(to_sq).zip(board.color_on(to_sq)) {
            acc.update::<false>(to_sq, captured, color);
        }

        if let Some(ep) = board.en_passant() {
            let (stm_fifth, stm_sixth) = match from_color {
                Color::White => (Rank::Fifth, Rank::Sixth),
                Color::Black => (Rank::Fourth, Rank::Third),
            };
            if from_type == Piece::Pawn && to_sq == Square::new(ep, stm_sixth) {
                acc.update::<false>(Square::new(ep, stm_fifth), Piece::Pawn, !from_color);
            }
        }
        if Some(from_color) == board.color_on(to_sq) {
            let stm_first = match from_color {
                Color::White => Rank::First,
                Color::Black => Rank::Eighth,
            };
            if to_sq.file() > from_sq.file() {
                acc.update::<true>(Square::new(File::G, stm_first), Piece::King, from_color);
                acc.update::<true>(Square::new(File::F, stm_first), Piece::Rook, from_color);
            } else {
                acc.update::<true>(Square::new(File::C, stm_first), Piece::King, from_color);
                acc.update::<true>(Square::new(File::D, stm_first), Piece::Rook, from_color);
            }
        } else {
            acc.update::<true>(to_sq, make_move.promotion.unwrap_or(from_type), from_color);
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
