use chess::{BitBoard, Board, Color, Piece, EMPTY};

use self::normal::{Dense, Incremental};

mod normal;

include!(concat!(env!("OUT_DIR"), "/nnue_weights.rs"));

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
    out_layer: Dense<'static, MID, OUTPUT>,
}

impl Nnue {
    pub fn new() -> Self {
        let input_layer = Incremental::new(&INCREMENTAL, INCREMENTAL_BIAS);
        let out_layer = Dense::new(&OUT, [0]);

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
            out_layer,
        }
    }

    pub fn feed_forward(&mut self, board: &Board) -> i16 {
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
                } else {
                    self.w_input_layer.incr_ff::<-1>(64 * w_index + w_sq);
                    self.b_input_layer.incr_ff::<-1>(64 * b_index + b_sq);
                }
            }
        }

        let w_incr_layer = *self.w_input_layer.get();
        let w_incr_layer = normal::clipped_relu(w_incr_layer);

        let b_incr_layer = *self.b_input_layer.get();
        let b_incr_layer = normal::clipped_relu(b_incr_layer);

        normal::out(
            self.out_layer.ff_sym(&w_incr_layer, &b_incr_layer)[0]
        )
    }
}
