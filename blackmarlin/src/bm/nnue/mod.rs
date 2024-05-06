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
    w_acc: Align<[i16; MID]>,
    b_acc: Align<[i16; MID]>,
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

#[derive(Debug, Clone)]
pub struct Nnue {
    accumulator: Vec<Accumulator>,
    bias: Arc<Align<[i16; MID]>>,
    head: usize,
    out_layer: Dense<{ MID * 2 }, OUTPUT>,

    w_input_layer: Incremental<INPUT, MID>,
    b_input_layer: Incremental<INPUT, MID>,

    w_add: ArrayVec<usize, 48>,
    b_add: ArrayVec<usize, 48>,
    w_rm: ArrayVec<usize, 48>,
    b_rm: ArrayVec<usize, 48>,

    null_moves: Vec<bool>,
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

        let input_layer = Incremental::new(incremental);
        let out_layer = Dense::new(out, out_bias);

        Self {
            accumulator: vec![
                Accumulator {
                    w_acc: incremental_bias,
                    b_acc: incremental_bias
                };
                ab_runner::MAX_PLY as usize + 1
            ],
            w_input_layer: input_layer.clone(),
            b_input_layer: input_layer,
            w_add: ArrayVec::new(),
            w_rm: ArrayVec::new(),
            b_add: ArrayVec::new(),
            b_rm: ArrayVec::new(),
            bias: Arc::new(Align(incremental_bias.0)),
            out_layer,
            head: 0,
            null_moves: Vec::with_capacity(ab_runner::MAX_PLY as usize + 1),
        }
    }

    pub fn perform_reset_update(&mut self, color: Color) {
        let curr = &mut self.accumulator[self.head];
        match color {
            Color::White => self.w_input_layer.update_features(
                &self.bias,
                &mut curr.w_acc,
                &self.w_add,
                &self.w_rm,
            ),
            Color::Black => self.b_input_layer.update_features(
                &self.bias,
                &mut curr.b_acc,
                &self.b_add,
                &self.b_rm,
            ),
        }
    }

    pub fn perform_update(&mut self, color: Color) {
        let (prev, curr) = self.accumulator.split_at_mut(self.head);
        let prev = prev.last().unwrap();
        match color {
            Color::White => self.w_input_layer.update_features(
                &prev.w_acc,
                &mut curr[0].w_acc,
                &self.w_add,
                &self.w_rm,
            ),
            Color::Black => self.b_input_layer.update_features(
                &prev.b_acc,
                &mut curr[0].b_acc,
                &self.b_add,
                &self.b_rm,
            ),
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
            Color::White => acc.w_acc = *self.bias,
            Color::Black => acc.b_acc = *self.bias,
        }
        self.perform_reset_update(perspective);
        self.clear();
    }

    pub fn full_reset(&mut self, board: &Board, w_threats: BitBoard, b_threats: BitBoard) {
        self.head = 0;
        self.reset(Color::White, board, w_threats, b_threats);
        self.reset(Color::Black, board, w_threats, b_threats);
    }

    fn push_accumulator(&mut self) {
        self.null_moves.push(false);
        self.head += 1;
    }

    pub fn null_move(&mut self) {
        self.null_moves.push(true);
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
            self.perform_update(perspective);
        }
        self.clear();
    }

    pub fn unmake_move(&mut self) {
        let null_move = self.null_moves.pop().unwrap();
        if !null_move {
            self.head -= 1;
        }
    }

    pub fn feed_forward(&mut self, stm: Color, piece_cnt: usize) -> i16 {
        let acc = &mut self.accumulator[self.head];
        let mut incr = Align([0; MID * 2]);
        let (stm, nstm) = match stm {
            Color::White => (&acc.w_acc, &acc.b_acc),
            Color::Black => (&acc.b_acc, &acc.w_acc),
        };
        layers::sq_clipped_relu(stm, &mut incr.0);
        layers::sq_clipped_relu(nstm, &mut incr.0[MID..]);

        let bucket = (((63 - piece_cnt) * (32 - piece_cnt)) / 225).min(7);
        layers::scale_network_output(self.out_layer.feed_forward(&incr, bucket))
    }
}
