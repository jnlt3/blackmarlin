use chess::{BitBoard, Board, Color, Piece, EMPTY};

use self::normal::{Dense, Incremental, Psqt};

mod normal;

include!(concat!(env!("OUT_DIR"), "/nnue_weights.rs"));

const TEMPO: f32 = 0.2;
const STM_WEIGHT: f32 = (1.0 + TEMPO) * 0.5;

#[derive(Debug, Clone)]
pub struct Nnue {
    white: BitBoard,
    black: BitBoard,
    pawns: BitBoard,
    knights: BitBoard,
    bishops: BitBoard,
    rooks: BitBoard,
    queens: BitBoard,
    kings: BitBoard,

    inputs: [[i8; 64]; 12],

    w_input_layer: Incremental<'static, INPUT, MID>,
    b_input_layer: Incremental<'static, INPUT, MID>,
    w_res_layer: Psqt<'static, INPUT, OUTPUT>,
    b_res_layer: Psqt<'static, INPUT, OUTPUT>,
    out_layer: Dense<'static, MID, OUTPUT>,
}

impl Nnue {
    pub fn new() -> Self {
        let input_layer = Incremental::new(&INCREMENTAL, INCREMENTAL_BIAS);
        let res_layer = Psqt::new(&PSQT);
        let out_layer = Dense::new(&OUT, OUT_BIAS);

        Self {
            white: EMPTY,
            black: EMPTY,
            pawns: EMPTY,
            knights: EMPTY,
            bishops: EMPTY,
            rooks: EMPTY,
            queens: EMPTY,
            kings: EMPTY,

            inputs: [[0_i8; 64]; 12],
            w_input_layer: input_layer.clone(),
            b_input_layer: input_layer,
            w_res_layer: res_layer.clone(),
            b_res_layer: res_layer,
            out_layer,
        }
    }

    #[inline]
    pub fn feed_forward(&mut self, board: &Board, bucket: usize) -> i16 {
        let white = *board.color_combined(Color::White);
        let black = *board.color_combined(Color::Black);

        let pawns = *board.pieces(Piece::Pawn);
        let knights = *board.pieces(Piece::Knight);
        let bishops = *board.pieces(Piece::Bishop);
        let rooks = *board.pieces(Piece::Rook);
        let queens = *board.pieces(Piece::Queen);
        let kings = *board.pieces(Piece::King);

        let array = [
            (white & pawns) ^ (self.white & self.pawns),
            (white & knights) ^ (self.white & self.knights),
            (white & bishops) ^ (self.white & self.bishops),
            (white & rooks) ^ (self.white & self.rooks),
            (white & queens) ^ (self.white & self.queens),
            (white & kings) ^ (self.white & self.kings),
            (black & pawns) ^ (self.black & self.pawns),
            (black & knights) ^ (self.black & self.knights),
            (black & bishops) ^ (self.black & self.bishops),
            (black & rooks) ^ (self.black & self.rooks),
            (black & queens) ^ (self.black & self.queens),
            (black & kings) ^ (self.black & self.kings),
        ];

        self.white = white;
        self.black = black;
        self.pawns = pawns;
        self.knights = knights;
        self.bishops = bishops;
        self.rooks = rooks;
        self.queens = queens;
        self.kings = kings;

        for (w_index, (input, &bb)) in self.inputs.iter_mut().zip(&array).enumerate() {
            let b_index = (w_index + 6) % 12;
            for w_sq in bb {
                let w_sq = w_sq.to_index();
                let b_sq = w_sq ^ 56;

                let input = &mut input[w_sq];
                let old = *input;
                let new = 1 - old;
                *input = new;

                if new == 1 {
                    self.w_input_layer.incr_ff::<1>(64 * w_index + w_sq);
                    self.b_input_layer.incr_ff::<1>(64 * b_index + b_sq);
                    self.w_res_layer.incr_ff::<1>(64 * w_index + w_sq);
                    self.b_res_layer.incr_ff::<1>(64 * b_index + b_sq);
                } else {
                    self.w_input_layer.incr_ff::<-1>(64 * w_index + w_sq);
                    self.b_input_layer.incr_ff::<-1>(64 * b_index + b_sq);
                    self.w_res_layer.incr_ff::<-1>(64 * w_index + w_sq);
                    self.b_res_layer.incr_ff::<-1>(64 * b_index + b_sq);
                }
            }
        }

        let w_incr_layer = *self.w_input_layer.get();
        let w_incr_layer = normal::clipped_relu(w_incr_layer);

        let b_incr_layer = *self.b_input_layer.get();
        let b_incr_layer = normal::clipped_relu(b_incr_layer);

        let w_res_out = self.w_res_layer.get()[bucket] as f32;
        let b_res_out = self.b_res_layer.get()[bucket] as f32;

        let w_out = self.out_layer.ff(&w_incr_layer, bucket)[bucket] as f32;
        let b_out = self.out_layer.ff(&b_incr_layer, bucket)[bucket] as f32;

        let (w_out, b_out, w_res_out, b_res_out) = match board.side_to_move() {
            Color::White => (w_out, b_out, w_res_out, b_res_out),
            Color::Black => (b_out, w_out, b_res_out, w_res_out),
        };

        let out = w_out * STM_WEIGHT - b_out * (1.0 - STM_WEIGHT);
        let res_out = (w_res_out * STM_WEIGHT - b_res_out * (1.0 - STM_WEIGHT)) / 64.0;
        res_out as i16 + normal::out(out)
    }
}
