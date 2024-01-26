use std::sync::Arc;

use arrayvec::ArrayVec;
use cozy_chess::{Board, Color, File, Move, Piece, Rank, Square};

use self::layers::{Align, Dense, Incremental};

use super::{
    bm_runner::ab_runner,
    bm_util::threats::{self, Threats},
};

mod include;
mod layers;

include!(concat!(env!("OUT_DIR"), "/arch.rs"));

const NN_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eval.bin"));
#[derive(Debug, Clone)]
pub struct Accumulator {
    w_input_layer: Incremental<INPUT, MID>,
    b_input_layer: Incremental<INPUT, MID>,
}

fn king_to_index(sq: Square) -> usize {
    sq.file() as usize * Rank::NUM + sq.rank() as usize
}

fn halfka_feature(
    perspective: Color,
    king: Square,
    color: Color,
    piece: Piece,
    square: Square,
    is_threat: bool,
) -> usize {
    let (mut king, mut square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    if king.file() > File::D {
        king = king.flip_file();
        square = square.flip_file();
    };
    let piece_idx = if is_threat {
        match piece {
            Piece::Knight => Piece::NUM,
            Piece::Bishop => Piece::NUM + 1,
            Piece::Rook => Piece::NUM + 2,
            Piece::Queen => Piece::NUM + 3,
            _ => unreachable!(),
        }
    } else {
        piece as usize
    };
    let mut index = 0;
    index = index * Square::NUM / 2 + king_to_index(king);
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 4) + piece_idx as usize;
    index = index * Square::NUM + square as usize;
    index
}

#[derive(Debug, Clone, Copy)]
struct Update {
    w_index: usize,
    b_index: usize,
}

impl Update {
    fn new(w_index: usize, b_index: usize) -> Self {
        Self { w_index, b_index }
    }
}

fn indices(
    w_king: Square,
    b_king: Square,
    sq: Square,
    piece: Piece,
    color: Color,
    is_threat: bool,
) -> Update {
    let w_index = halfka_feature(Color::White, w_king, color, piece, sq, is_threat);
    let b_index = halfka_feature(Color::Black, b_king, color, piece, sq, is_threat);
    Update::new(w_index, b_index)
}

impl Accumulator {
    pub fn perform_update(
        &mut self,
        w_add: &mut [usize],
        w_rm: &mut [usize],
        b_add: &mut [usize],
        b_rm: &mut [usize],
    ) {
        self.w_input_layer.update_features(w_add, w_rm);
        self.b_input_layer.update_features(b_add, b_rm);
    }
}

#[derive(Debug, Clone)]
pub struct Nnue {
    accumulator: Vec<Accumulator>,
    bias: Arc<[i16; MID]>,
    head: usize,
    out_layer: Dense<{ MID * 2 }, OUTPUT>,

    w_add: ArrayVec<usize, 48>,
    b_add: ArrayVec<usize, 48>,
    w_rm: ArrayVec<usize, 48>,
    b_rm: ArrayVec<usize, 48>,
}

impl Nnue {
    pub fn new() -> Self {
        let mut bytes = &NN_BYTES[12..];
        let incremental = Arc::from(include::sparse_from_bytes_i16::<INPUT, MID>(bytes));
        bytes = &bytes[INPUT * MID * 2..];
        let incremental_bias = include::bias_from_bytes_i16::<i16, MID>(bytes);
        bytes = &bytes[MID * 2..];
        let out = Arc::from(include::dense_from_bytes_i8::<i8, { MID * 2 }, OUTPUT>(
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
            w_add: ArrayVec::new(),
            w_rm: ArrayVec::new(),
            b_add: ArrayVec::new(),
            b_rm: ArrayVec::new(),
            bias: Arc::new(incremental_bias.0),
            out_layer,
            head: 0,
        }
    }

    fn update<const INCR: bool>(&mut self, update: Update) {
        match INCR {
            true => {
                self.w_add.push(update.w_index);
                self.b_add.push(update.b_index);
            }
            false => {
                self.w_rm.push(update.w_index);
                self.b_rm.push(update.b_index);
            }
        }
    }

    fn clear(&mut self) {
        self.w_add.clear();
        self.w_rm.clear();
        self.b_add.clear();
        self.b_rm.clear();
    }

    pub fn reset(&mut self, board: &Board, threats: Threats) {
        let w_king = board.king(Color::White);
        let b_king = board.king(Color::Black);

        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            self.update::<true>(indices(w_king, b_king, sq, piece, color, false));
            if threats.from_color(!color).has(sq) {
                self.update::<true>(indices(w_king, b_king, sq, piece, color, true));
            }
        }

        let acc = &mut self.accumulator[self.head];
        acc.w_input_layer.reset(*self.bias);
        acc.b_input_layer.reset(*self.bias);
        acc.perform_update(
            &mut self.w_add,
            &mut self.w_rm,
            &mut self.b_add,
            &mut self.b_rm,
        );
        self.clear();
    }

    pub fn full_reset(&mut self, board: &Board, threats: Threats) {
        self.head = 0;
        self.reset(board, threats);
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

    pub fn make_move(
        &mut self,
        board: &Board,
        make_move: Move,
        threats: Threats,
        old_threats: Threats,
    ) {
        self.push_accumulator();
        let from_sq = make_move.from;
        let from_type = board.piece_on(from_sq).unwrap();
        let stm = board.side_to_move();
        let w_king = board.king(Color::White);
        let b_king = board.king(Color::Black);
        if from_type == Piece::King {
            let mut board_clone = board.clone();
            board_clone.play_unchecked(make_move);
            self.reset(&board_clone, threats);
            return;
        }

        for color in Color::ALL {
            for piece in threats::PIECES {
                let current = threats.to_piece(color, piece);
                let diff = current ^ old_threats.to_piece(color, piece);
                for sq in diff & current {
                    self.update::<true>(indices(w_king, b_king, sq, piece, color, true));
                }
                for sq in diff & !current {
                    self.update::<false>(indices(w_king, b_king, sq, piece, color, true));
                }
            }
        }

        self.update::<false>(indices(w_king, b_king, from_sq, from_type, stm, false));

        let to_sq = make_move.to;
        if let Some((captured, color)) = board.piece_on(to_sq).zip(board.color_on(to_sq)) {
            self.update::<false>(indices(w_king, b_king, to_sq, captured, color, false));
        }

        if let Some(ep) = board.en_passant() {
            let (stm_fifth, stm_sixth) = match stm {
                Color::White => (Rank::Fifth, Rank::Sixth),
                Color::Black => (Rank::Fourth, Rank::Third),
            };
            if from_type == Piece::Pawn && to_sq == Square::new(ep, stm_sixth) {
                self.update::<false>(indices(
                    w_king,
                    b_king,
                    Square::new(ep, stm_fifth),
                    Piece::Pawn,
                    !stm,
                    false,
                ));
            }
        }
        if Some(stm) == board.color_on(to_sq) {
            let stm_first = match stm {
                Color::White => Rank::First,
                Color::Black => Rank::Eighth,
            };
            if to_sq.file() > from_sq.file() {
                self.update::<true>(indices(
                    w_king,
                    b_king,
                    Square::new(File::G, stm_first),
                    Piece::King,
                    stm,
                    false,
                ));
                self.update::<true>(indices(
                    w_king,
                    b_king,
                    Square::new(File::F, stm_first),
                    Piece::Rook,
                    stm,
                    false,
                ));
            } else {
                self.update::<true>(indices(
                    w_king,
                    b_king,
                    Square::new(File::C, stm_first),
                    Piece::King,
                    stm,
                    false,
                ));
                self.update::<true>(indices(
                    w_king,
                    b_king,
                    Square::new(File::D, stm_first),
                    Piece::Rook,
                    stm,
                    false,
                ));
            }
        } else {
            self.update::<true>(indices(
                w_king,
                b_king,
                to_sq,
                make_move.promotion.unwrap_or(from_type),
                stm,
                false,
            ));
        }
        self.accumulator[self.head].perform_update(
            &mut self.w_add,
            &mut self.w_rm,
            &mut self.b_add,
            &mut self.b_rm,
        );
        self.clear();
    }

    pub fn unmake_move(&mut self) {
        self.head -= 1;
    }

    pub fn feed_forward(&mut self, stm: Color, piece_cnt: usize) -> i16 {
        let acc = &mut self.accumulator[self.head];
        let mut incr = Align([0; MID * 2]);
        let (stm, nstm) = match stm {
            Color::White => (&acc.w_input_layer, &acc.b_input_layer),
            Color::Black => (&acc.b_input_layer, &acc.w_input_layer),
        };
        layers::sq_clipped_relu(*stm.get(), &mut incr.0);
        layers::sq_clipped_relu(*nstm.get(), &mut incr.0[MID..]);

        let bucket = (piece_cnt / 4).min(7);
        layers::scale_network_output(self.out_layer.feed_forward(&incr, bucket))
    }
}
