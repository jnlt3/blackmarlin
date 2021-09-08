use chess::{Board, ChessMove, MoveGen, EMPTY};

use crate::bm::bm_runner::ab_runner::SearchOptions;
use crate::bm::bm_search::move_gen::GenType::{PvMove, Quiet};

use crate::bm::bm_util::c_hist::{CMoveHistoryTable, PieceTo};

use crate::bm::bm_util::ch_table::CaptureHistoryTable;
use crate::bm::bm_util::evaluator::Evaluator;
use crate::bm::bm_util::h_table::HistoryTable;
use std::marker::PhantomData;
use std::sync::Arc;

use super::move_entry::MoveEntryIterator;

//TODO: don't allocate on the heap when it is not needed

const COUNTER_MOVE_BONUS: u32 = 0;
const C_HIST_FACTOR: i32 = 0;
const C_HIST_DIVISOR: i32 = 8;
const CH_TABLE_FACTOR: i32 = 1;
const CH_TABLE_DIVISOR: i32 = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GenType {
    PvMove,
    CalcCaptures,
    Captures,
    GenQuiet,
    ThreatMove,
    Killer,
    CalcQuiet,
    Quiet,
    BadCaptures,
}

pub struct PvMoveGen {
    move_gen: MoveGen,
    board: Board,
    pv_move: Option<ChessMove>,
    gen_type: GenType,
}

impl PvMoveGen {
    pub fn new(board: &Board, pv_move: Option<ChessMove>) -> Self {
        Self {
            gen_type: PvMove,
            move_gen: MoveGen::new_legal(board),
            board: *board,
            pv_move,
        }
    }
}

impl Iterator for PvMoveGen {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gen_type == PvMove {
            self.gen_type = Quiet;
            if let Some(pv_move) = self.pv_move {
                if self.board.legal(pv_move) {
                    self.move_gen.remove_move(pv_move);
                    return Some(pv_move);
                }
            }
        }
        self.move_gen.next()
    }
}

pub struct OrderedMoveGen<Eval: Evaluator, const T: usize, const K: usize> {
    move_gen: MoveGen,
    pv_move: Option<ChessMove>,
    threat_move_entry: MoveEntryIterator<T>,
    killer_entry: MoveEntryIterator<K>,
    h_table: Arc<HistoryTable>,
    ch_table: Arc<CaptureHistoryTable>,
    c_hist: Arc<CMoveHistoryTable>,
    counter_move: Option<ChessMove>,
    prev_move: Option<PieceTo>,
    gen_type: GenType,
    board: Board,

    capture_queue: Vec<(ChessMove, i32)>,
    quiet_queue: Vec<(ChessMove, u32)>,
    bad_capture_queue: Vec<(ChessMove, i32)>,
    eval: PhantomData<Eval>,
}

impl<Eval: 'static + Evaluator + Clone + Send, const T: usize, const K: usize>
    OrderedMoveGen<Eval, T, K>
{
    pub fn new(
        board: &Board,
        pv_move: Option<ChessMove>,
        threat_move_entry: MoveEntryIterator<T>,
        killer_entry: MoveEntryIterator<K>,
        options: &SearchOptions<Eval>,
        prev_move: Option<PieceTo>,
    ) -> Self {
        let mut counter_move = None;
        if let Some(piece_to) = prev_move {
            counter_move =
                options
                    .get_c_table()
                    .get(!board.side_to_move(), piece_to.piece, piece_to.to);
        }
        Self {
            gen_type: GenType::PvMove,
            move_gen: MoveGen::new_legal(board),
            pv_move,
            threat_move_entry,
            killer_entry,
            h_table: options.get_h_table(),
            ch_table: options.get_ch_table(),
            counter_move,
            prev_move,
            c_hist: options.get_c_hist(),
            board: *board,
            capture_queue: vec![],
            quiet_queue: vec![],
            bad_capture_queue: vec![],
            eval: PhantomData::default(),
        }
    }
}

impl<Eval: Evaluator, const K: usize, const T: usize> Iterator for OrderedMoveGen<Eval, K, T> {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gen_type == GenType::PvMove {
            self.gen_type = GenType::CalcCaptures;
            #[cfg(feature = "pv")]
            if let Some(pv_move) = self.pv_move {
                if self.board.legal(pv_move) {
                    self.move_gen.remove_move(pv_move);
                    return Some(pv_move);
                }
            }
        }
        if self.gen_type == GenType::CalcCaptures {
            self.move_gen.set_iterator_mask(*self.board.combined());
            for make_move in &mut self.move_gen {
                let mut expected_gain = Eval::see(self.board, make_move);
                let queue = if expected_gain < 0 {
                    &mut self.bad_capture_queue
                } else {
                    &mut self.capture_queue
                };

                #[cfg(feature = "cm_table")]
                {
                    let history_gain = self.ch_table.get(
                        self.board.side_to_move(),
                        self.board.piece_on(make_move.get_source()).unwrap(),
                        make_move.get_dest(),
                        self.board.piece_on(make_move.get_dest()).unwrap(),
                    ) as i32
                        * CH_TABLE_FACTOR
                        / CH_TABLE_DIVISOR;
                    expected_gain += history_gain;
                }
                let pos = queue
                    .binary_search_by_key(&expected_gain, |(_, score)| *score)
                    .unwrap_or_else(|pos| pos);
                queue.insert(pos, (make_move, expected_gain));
            }
            self.gen_type = GenType::Captures;
        }
        if self.gen_type == GenType::Captures {
            if let Some((make_move, _)) = self.capture_queue.pop() {
                return Some(make_move);
            }
            self.gen_type = GenType::GenQuiet;
        }
        if self.gen_type == GenType::GenQuiet {
            self.move_gen.set_iterator_mask(!EMPTY);
            for make_move in &mut self.move_gen {
                //Later to be replaced by the actual value for sorting
                self.quiet_queue.push((make_move, 0));
            }
            self.gen_type = GenType::Killer;
        }
        //Assumes Killer Moves won't repeat
        if self.gen_type == GenType::Killer {
            #[cfg(feature = "killer")]
            for make_move in &mut self.killer_entry {
                if Some(make_move) != self.pv_move {
                    let position = self
                        .quiet_queue
                        .iter()
                        .position(|(cmp_move, _)| make_move == *cmp_move);
                    if let Some(position) = position {
                        self.quiet_queue.remove(position);
                        return Some(make_move);
                    }
                }
            }
            self.gen_type = GenType::ThreatMove;
        }
        if self.gen_type == GenType::ThreatMove {
            #[cfg(feature = "threat")]
            for make_move in &mut self.threat_move_entry {
                if Some(make_move) != self.pv_move {
                    let position = self
                        .quiet_queue
                        .iter()
                        .position(|(cmp_move, _)| make_move == *cmp_move);
                    if let Some(position) = position {
                        self.quiet_queue.remove(position);
                        return Some(make_move);
                    }
                }
            }
            self.gen_type = GenType::CalcQuiet;
        }
        if self.gen_type == GenType::CalcQuiet {
            for (make_move, score) in &mut self.quiet_queue {
                let piece = self.board.piece_on(make_move.get_source()).unwrap();

                #[cfg(feature = "hist")]
                {
                    *score =
                        self.h_table
                            .get(self.board.side_to_move(), piece, make_move.get_dest());
                }

                #[cfg(feature = "cm_table")]
                {
                    if Some(*make_move) == self.counter_move {
                        *score += COUNTER_MOVE_BONUS;
                    }
                }
                #[cfg(feature = "cm_hist")]
                if let Some(last_move) = &self.prev_move {
                    let counter_move_hist = self.c_hist.get(
                        !self.board.side_to_move(),
                        last_move.piece,
                        last_move.to,
                        self.board.piece_on(make_move.get_source()).unwrap(),
                        make_move.get_dest(),
                    ) * C_HIST_FACTOR as u32
                        / C_HIST_DIVISOR as u32;
                    *score += counter_move_hist;
                }
            }
            self.quiet_queue.sort_unstable_by_key(|(_, score)| *score);
            self.gen_type = GenType::Quiet;
        }
        if self.gen_type == GenType::Quiet {
            if let Some((make_move, _)) = self.quiet_queue.pop() {
                return Some(make_move);
            }
            self.gen_type = GenType::BadCaptures;
        }
        if let Some((make_move, _)) = self.bad_capture_queue.pop() {
            return Some(make_move);
        }
        None
    }
}
