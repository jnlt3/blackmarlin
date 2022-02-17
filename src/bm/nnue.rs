use cozy_chess::{BitBoard, Board, Color, Piece};

use self::normal::{Dense, Incremental, Psqt};

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
    w_res_layer: Psqt<'static, INPUT, OUTPUT>,
    b_res_layer: Psqt<'static, INPUT, OUTPUT>,
    w_s_res_layer: Psqt<'static, INPUT, OUTPUT>,
    b_s_res_layer: Psqt<'static, INPUT, OUTPUT>,
    out_layer: Dense<'static, MID, OUTPUT>,
    s_out_layer: Dense<'static, MID, OUTPUT>,
}

impl Nnue {
    pub fn new() -> Self {
        let input_layer = Incremental::new(&INCREMENTAL, INCREMENTAL_BIAS);
        let res_layer = Psqt::new(&PSQT);
        let s_res_layer = Psqt::new(&S_PSQT);
        let out_layer = Dense::new(&OUT, OUT_BIAS);
        let s_out_layer = Dense::new(&S_OUT, S_OUT_BIAS);

        Self {
            white: BitBoard::EMPTY,
            black: BitBoard::EMPTY,
            pawns: BitBoard::EMPTY,
            knights: BitBoard::EMPTY,
            bishops: BitBoard::EMPTY,
            rooks: BitBoard::EMPTY,
            queens: BitBoard::EMPTY,
            kings: BitBoard::EMPTY,

            inputs: [[0_i8; 64]; 12],
            w_input_layer: input_layer.clone(),
            b_input_layer: input_layer,
            w_res_layer: res_layer.clone(),
            b_res_layer: res_layer,
            w_s_res_layer: s_res_layer.clone(),
            b_s_res_layer: s_res_layer,
            out_layer,
            s_out_layer,
        }
    }

    #[inline]
    pub fn feed_forward(&mut self, board: &Board, bucket: usize) -> i16 {
        let white = board.colors(Color::White);
        let black = board.colors(Color::Black);

        let pawns = board.pieces(Piece::Pawn);
        let knights = board.pieces(Piece::Knight);
        let bishops = board.pieces(Piece::Bishop);
        let rooks = board.pieces(Piece::Rook);
        let queens = board.pieces(Piece::Queen);
        let kings = board.pieces(Piece::King);

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
                let w_sq = w_sq as usize;
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
                    self.w_s_res_layer.incr_ff::<1>(64 * w_index + w_sq);
                    self.b_s_res_layer.incr_ff::<1>(64 * b_index + b_sq);
                } else {
                    self.w_input_layer.incr_ff::<-1>(64 * w_index + w_sq);
                    self.b_input_layer.incr_ff::<-1>(64 * b_index + b_sq);
                    self.w_res_layer.incr_ff::<-1>(64 * w_index + w_sq);
                    self.b_res_layer.incr_ff::<-1>(64 * b_index + b_sq);
                    self.w_s_res_layer.incr_ff::<-1>(64 * w_index + w_sq);
                    self.b_s_res_layer.incr_ff::<-1>(64 * b_index + b_sq);
                }
            }
        }

        let (incr_layer, psqt_score, s_psqt_score) = match board.side_to_move() {
            Color::White => (
                normal::clipped_relu(*self.w_input_layer.get()),
                self.w_res_layer.get()[bucket] / 64,
                self.w_s_res_layer.get()[bucket] / 64,
            ),
            Color::Black => (
                normal::clipped_relu(*self.b_input_layer.get()),
                self.b_res_layer.get()[bucket] / 64,
                self.b_s_res_layer.get()[bucket] / 64,
            ),
        };
        let eval = psqt_score as i16 + normal::out(self.out_layer.ff(&incr_layer, bucket)[bucket]);
        let scale = s_psqt_score as i16 + normal::out(self.s_out_layer.ff(&incr_layer, bucket)[bucket]);
        ((eval as i32 * scale as i32) / normal::UNITS as i32) as i16
    }
}