use chess::{BitBoard, Board, Color, Piece, EMPTY};

use self::normal::{Dense, Incremental, NonConstWeights};
use serde::{Deserialize, Serialize};

mod normal;

const INPUT: usize = 768;
const MID_0: usize = 128;
const MID_1: usize = 64;
const MID_2: usize = 32;
const OUTPUT: usize = 1;

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

    input_layer: Incremental<INPUT, MID_0>,
    mid_0_layer: Dense<MID_0, MID_1>,
    mid_1_layer: Dense<MID_1, MID_2>,
    out_layer: Dense<MID_2, OUTPUT>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct W {
    pub weights: Vec<Vec<Vec<f32>>>,
}

impl Nnue {
    pub fn new(file: String) -> Self {
        let weights = std::fs::read_to_string(file).unwrap();
        let mut weights = serde_json::from_str::<W>(&weights).unwrap().weights;
        let output_weights = NonConstWeights(weights.pop().unwrap());
        let mid_1_weights = NonConstWeights(weights.pop().unwrap());
        let mid_0_weights = NonConstWeights(weights.pop().unwrap());
        let input_weights = NonConstWeights(weights.pop().unwrap());
        let input_layer = Incremental::new(input_weights.into());
        let mid_0_layer = Dense::new(mid_0_weights.into());
        let mid_1_layer = Dense::new(mid_1_weights.into());
        let out_layer = Dense::new(output_weights.into());
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
            input_layer,
            mid_0_layer,
            mid_1_layer,
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

        for (index, (input, &bb)) in self.inputs.iter_mut().zip(&array).enumerate() {
            for sq in bb {
                let input = &mut input[sq.to_index()];
                let old = *input;
                let new = 1 - old;
                *input = new;
                self.input_layer.incr_ff(64 * index + sq.to_index(), new - old);
            }
        }

        let incr_layer = *self.input_layer.get();
        let mut incr_layer = normal::clipped_relu(incr_layer);
        
        let mut mid_0 = self.mid_0_layer.ff(&mut incr_layer);
        normal::scale(&mut mid_0);
        let mut mid_0 = normal::clipped_relu(mid_0);

        let mut mid_1 = self.mid_1_layer.ff(&mut mid_0);
        normal::scale(&mut mid_1);
        let mid_1 = normal::clipped_relu(mid_1);

        normal::out(self.out_layer.ff(&mid_1)[0])
    }
}
