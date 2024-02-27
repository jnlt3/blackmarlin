use std::sync::Arc;

use arrayvec::ArrayVec;
use cozy_chess::{BitBoard, Board, Color, File, Move, Piece, Rank, Square};

use self::layers::{Align, Dense, Incremental};

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

fn king_to_index(sq: Square) -> usize {
    sq.file() as usize * Rank::NUM + sq.rank() as usize
}

fn halfka_feature(
    perspective: Color,
    king: Square,
    color: Color,
    piece: Piece,
    square: Square,
) -> usize {
    let (mut king, mut square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    if king.file() > File::D {
        king = king.flip_file();
        square = square.flip_file();
    };
    let mut index = 0;
    index = index * Square::NUM / 2 + king_to_index(king);
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 1) + piece as usize;
    index = index * Square::NUM + square as usize;
    index
}

fn threat_feature(perspective: Color, king: Square, color: Color, square: Square) -> usize {
    let (mut king, mut square, color) = match perspective {
        Color::White => (king, square, color),
        Color::Black => (king.flip_rank(), square.flip_rank(), !color),
    };
    if king.file() > File::D {
        king = king.flip_file();
        square = square.flip_file();
    }
    let mut index = 0;
    index = index * Square::NUM / 2 + king_to_index(king);
    index = index * Color::NUM + color as usize;
    index = index * (Piece::NUM + 1) + Piece::NUM;
    index = index * Square::NUM + square as usize;
    index
}

#[derive(Debug, Clone, Copy)]
struct Update {
    index: usize,
    color: Color,
}

impl Update {
    fn new(index: usize, color: Color) -> Self {
        Self { index, color }
    }
}

fn piece_indices(
    perspective: Color,
    king: Square,
    sq: Square,
    piece: Piece,
    color: Color,
) -> Update {
    let index = halfka_feature(perspective, king, color, piece, sq);
    Update::new(index, perspective)
}

fn threat_indices(perspective: Color, king: Square, sq: Square, color: Color) -> Update {
    let index = threat_feature(perspective, king, color, sq);
    Update::new(index, perspective)
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
            true => match update.color {
                Color::White => self.w_add.push(update.index),
                Color::Black => self.b_add.push(update.index),
            },
            false => match update.color {
                Color::White => self.w_rm.push(update.index),
                Color::Black => self.b_rm.push(update.index),
            },
        }
    }

    fn clear(&mut self) {
        self.w_add.clear();
        self.w_rm.clear();
        self.b_add.clear();
        self.b_rm.clear();
    }

    pub fn reset(
        &mut self,
        perspective: Color,
        board: &Board,
        w_threats: BitBoard,
        b_threats: BitBoard,
    ) {
        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            self.update::<true>(piece_indices(
                perspective,
                board.king(perspective),
                sq,
                piece,
                color,
            ));
        }
        for sq in w_threats {
            self.update::<true>(threat_indices(
                perspective,
                board.king(perspective),
                sq,
                Color::Black,
            ));
        }
        for sq in b_threats {
            self.update::<true>(threat_indices(
                perspective,
                board.king(perspective),
                sq,
                Color::White,
            ));
        }

        let acc = &mut self.accumulator[self.head];
        match perspective {
            Color::White => acc.w_input_layer.reset(*self.bias),
            Color::Black => acc.b_input_layer.reset(*self.bias),
        }
        acc.perform_update(
            &mut self.w_add,
            &mut self.w_rm,
            &mut self.b_add,
            &mut self.b_rm,
        );
        self.clear();
    }

    pub fn full_reset(&mut self, board: &Board, w_threats: BitBoard, b_threats: BitBoard) {
        self.head = 0;
        self.reset(Color::White, board, w_threats, b_threats);
        self.reset(Color::Black, board, w_threats, b_threats);
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
        new_board: &Board,
        make_move: Move,
        w_threats: BitBoard,
        b_threats: BitBoard,
        old_w_threats: BitBoard,
        old_b_threats: BitBoard,
    ) {
        self.push_accumulator();
        let from_sq = make_move.from;
        let from_type = board.piece_on(from_sq).unwrap();
        let stm = board.side_to_move();
        let mut perspectives: &[Color] = &[Color::White, Color::Black];
        let single = &[!stm];
        if from_type == Piece::King {
            perspectives = single;
            self.reset(stm, &new_board, w_threats, b_threats);
        }
        for &perspective in perspectives {
            for w_threat_sq in w_threats ^ old_w_threats {
                match w_threats.has(w_threat_sq) {
                    true => self.update::<true>(threat_indices(
                        perspective,
                        board.king(perspective),
                        w_threat_sq,
                        Color::Black,
                    )),
                    false => self.update::<false>(threat_indices(
                        perspective,
                        board.king(perspective),
                        w_threat_sq,
                        Color::Black,
                    )),
                };
            }

            for b_threat_sq in b_threats ^ old_b_threats {
                match b_threats.has(b_threat_sq) {
                    true => self.update::<true>(threat_indices(
                        perspective,
                        board.king(perspective),
                        b_threat_sq,
                        Color::White,
                    )),
                    false => self.update::<false>(threat_indices(
                        perspective,
                        board.king(perspective),
                        b_threat_sq,
                        Color::White,
                    )),
                }
            }

            self.update::<false>(piece_indices(
                perspective,
                board.king(perspective),
                from_sq,
                from_type,
                stm,
            ));

            let to_sq = make_move.to;
            if let Some((captured, color)) = board.piece_on(to_sq).zip(board.color_on(to_sq)) {
                self.update::<false>(piece_indices(
                    perspective,
                    board.king(perspective),
                    to_sq,
                    captured,
                    color,
                ));
            }

            if let Some(ep) = board.en_passant() {
                let (stm_fifth, stm_sixth) = match stm {
                    Color::White => (Rank::Fifth, Rank::Sixth),
                    Color::Black => (Rank::Fourth, Rank::Third),
                };
                if from_type == Piece::Pawn && to_sq == Square::new(ep, stm_sixth) {
                    self.update::<false>(piece_indices(
                        perspective,
                        board.king(perspective),
                        Square::new(ep, stm_fifth),
                        Piece::Pawn,
                        !stm,
                    ));
                }
            }
            if Some(stm) == board.color_on(to_sq) {
                let stm_first = match stm {
                    Color::White => Rank::First,
                    Color::Black => Rank::Eighth,
                };
                if to_sq.file() > from_sq.file() {
                    self.update::<true>(piece_indices(
                        perspective,
                        board.king(perspective),
                        Square::new(File::G, stm_first),
                        Piece::King,
                        stm,
                    ));
                    self.update::<true>(piece_indices(
                        perspective,
                        board.king(perspective),
                        Square::new(File::F, stm_first),
                        Piece::Rook,
                        stm,
                    ));
                } else {
                    self.update::<true>(piece_indices(
                        perspective,
                        board.king(perspective),
                        Square::new(File::C, stm_first),
                        Piece::King,
                        stm,
                    ));
                    self.update::<true>(piece_indices(
                        perspective,
                        board.king(perspective),
                        Square::new(File::D, stm_first),
                        Piece::Rook,
                        stm,
                    ));
                }
            } else {
                self.update::<true>(piece_indices(
                    perspective,
                    board.king(perspective),
                    to_sq,
                    make_move.promotion.unwrap_or(from_type),
                    stm,
                ));
            }
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

        let bucket = (((63 - piece_cnt) * (32 - piece_cnt)) / 225).min(7);
        layers::scale_network_output(self.out_layer.feed_forward(&incr, bucket))
    }
}
