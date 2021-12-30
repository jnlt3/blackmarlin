use cozy_chess::{Board, Move, Piece, PieceMoves};

use crate::bm::bm_eval::evaluator::StdEvaluator;

use crate::bm::bm_util::h_table::{DoubleMoveHistory, HistoryTable};
use arrayvec::ArrayVec;

use super::move_entry::MoveEntryIterator;

const MAX_MOVES: usize = 218;
const THRESHOLD: i16 = -(2_i16.pow(10));
const LOSING_CAPTURE: i16 = -(2_i16.pow(12));

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GenType {
    PvMove,
    CalcCaptures,
    Captures,
    GenQuiet,
    CounterMove,
    Killer,
    ThreatMove,
    Quiet,
}

type LazySee = Option<i16>;

pub struct OrderedMoveGen<const T: usize, const K: usize> {
    move_list: ArrayVec<PieceMoves, 18>,
    moves_played: ArrayVec<Move, 4>,
    pv_move: Option<Move>,
    threat_move_entry: MoveEntryIterator<T>,
    killer_entry: MoveEntryIterator<K>,
    counter_move: Option<Move>,
    prev_move: Option<Move>,
    gen_type: GenType,
    board: Board,

    queue: ArrayVec<(Move, i16, LazySee), MAX_MOVES>,
}

impl<const T: usize, const K: usize> OrderedMoveGen<T, K> {
    pub fn new(
        board: &Board,
        pv_move: Option<Move>,
        counter_move: Option<Move>,
        prev_move: Option<Move>,
        threat_move_entry: MoveEntryIterator<T>,
        killer_entry: MoveEntryIterator<K>,
    ) -> Self {
        Self {
            gen_type: GenType::PvMove,
            move_list: ArrayVec::new(),
            counter_move,
            prev_move,
            pv_move,
            threat_move_entry,
            killer_entry,
            board: board.clone(),
            queue: ArrayVec::new(),
            moves_played: ArrayVec::new(),
        }
    }

    pub fn next(
        &mut self,
        hist: &HistoryTable,
        c_hist: &HistoryTable,
        cm_hist: &DoubleMoveHistory,
    ) -> Option<Move> {
        if self.gen_type == GenType::PvMove {
            self.gen_type = GenType::CalcCaptures;
            if let Some(pv_move) = self.pv_move {
                if self.board.is_legal(pv_move) {
                    self.moves_played.push(pv_move);
                    return Some(pv_move);
                }
                self.pv_move = None;
            }
        }
        if self.gen_type == GenType::CalcCaptures {
            self.board.generate_moves(|piece_moves| {
                self.move_list.push(piece_moves);
                false
            });
            for &piece_moves in &self.move_list {
                let mut piece_moves = piece_moves;
                piece_moves.to &= self.board.colors(!self.board.side_to_move());
                for make_move in piece_moves {
                    if Some(make_move) == self.pv_move {
                        continue;
                    }
                    let expected_gain =
                        c_hist.get(self.board.side_to_move(), piece_moves.piece, make_move.to)
                            + StdEvaluator::see::<1>(&self.board, make_move) * 32;
                    self.queue.push((make_move, expected_gain, None));
                }
            }

            self.gen_type = GenType::Captures;
        }
        if self.gen_type == GenType::Captures {
            let mut max = THRESHOLD;
            let mut best_index = None;
            for (index, (make_move, score, see)) in self.queue.iter_mut().enumerate() {
                if *score > max {
                    let see_score =
                        see.unwrap_or_else(|| StdEvaluator::see::<16>(&self.board, *make_move));
                    *see = Some(see_score);
                    if see_score < 0 {
                        *score += LOSING_CAPTURE;
                        continue;
                    }
                    max = *score;
                    best_index = Some(index);
                }
            }
            if let Some(index) = best_index {
                return Some(self.queue.swap_remove(index).0);
            } else {
                self.gen_type = GenType::Killer;
            }
        }
        //Assumes Killer Moves won't repeat
        if self.gen_type == GenType::Killer {
            if let Some(make_move) = self.killer_entry.next() {
                if !self.moves_played.contains(&make_move) && self.board.is_legal(make_move) {
                    self.moves_played.push(make_move);
                    return Some(make_move);
                }
            }
            self.gen_type = GenType::CounterMove;
        }
        if self.gen_type == GenType::CounterMove {
            self.gen_type = GenType::GenQuiet;
            if let Some(counter_move) = self.counter_move {
                if !self.moves_played.contains(&counter_move) && self.board.is_legal(counter_move) {
                    self.moves_played.push(counter_move);
                    return Some(counter_move);
                }
            }
        }
        if self.gen_type == GenType::GenQuiet {
            for &piece_moves in &self.move_list {
                let mut piece_moves = piece_moves;
                piece_moves.to &= !self.board.colors(!self.board.side_to_move());
                for make_move in piece_moves {
                    if self.moves_played.contains(&make_move) {
                        continue;
                    }
                    if let Some(piece) = make_move.promotion {
                        match piece {
                            cozy_chess::Piece::Queen => {
                                self.queue.push((make_move, i16::MAX, None));
                            }
                            _ => {
                                self.queue.push((make_move, i16::MIN, None));
                            }
                        };
                        continue;
                    }
                    let mut score = 0;
                    let piece = self.board.piece_on(make_move.from).unwrap();

                    score += hist.get(self.board.side_to_move(), piece, make_move.to);
                    if let Some(prev_move) = self.prev_move {
                        let prev_move_piece =
                            self.board.piece_on(prev_move.to).unwrap_or(Piece::King);
                        score += cm_hist.get(
                            self.board.side_to_move(),
                            prev_move_piece,
                            prev_move.to,
                            piece,
                            make_move.to,
                        );
                    }

                    self.queue.push((make_move, score, None));
                }
            }
            self.gen_type = GenType::Quiet;
        }
        if self.gen_type == GenType::ThreatMove {
            for make_move in &mut self.threat_move_entry {
                let position = self
                    .queue
                    .iter()
                    .position(|(cmp_move, _, _)| make_move == *cmp_move);
                if let Some(position) = position {
                    self.queue.swap_remove(position);
                    return Some(make_move);
                }
            }
            self.gen_type = GenType::Quiet;
        }
        if self.gen_type == GenType::Quiet {
            let mut max = 0;
            let mut best_index = None;
            for (index, &(_, score, _)) in self.queue.iter().enumerate() {
                if best_index.is_none() || score > max {
                    max = score;
                    best_index = Some(index);
                }
            }
            return if let Some(index) = best_index {
                Some(self.queue.swap_remove(index).0)
            } else {
                None
            };
        }
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QSearchGenType {
    CalcCaptures,
    Captures,
}

pub struct QuiescenceSearchMoveGen {
    board: Board,
    gen_type: QSearchGenType,
    queue: ArrayVec<(Move, i16, LazySee), MAX_MOVES>,
}

impl QuiescenceSearchMoveGen {
    pub fn new(board: &Board) -> Self {
        Self {
            board: board.clone(),
            gen_type: QSearchGenType::CalcCaptures,
            queue: ArrayVec::new(),
        }
    }

    pub fn next(&mut self, c_hist: &HistoryTable) -> Option<Move> {
        if self.gen_type == QSearchGenType::CalcCaptures {
            self.board.generate_moves(|mut piece_moves| {
                piece_moves.to &= self.board.colors(!self.board.side_to_move());
                for make_move in piece_moves {
                    let expected_gain =
                        c_hist.get(self.board.side_to_move(), piece_moves.piece, make_move.to)
                            + StdEvaluator::see::<1>(&self.board, make_move) * 32;
                    self.queue.push((make_move, expected_gain, None));
                }
                false
            });
            self.gen_type = QSearchGenType::Captures;
        }
        let mut max = 0;
        let mut best_index = None;
        for (index, (make_move, score, see)) in self.queue.iter_mut().enumerate() {
            if best_index.is_none() || *score > max {
                let see_score =
                    see.unwrap_or_else(|| StdEvaluator::see::<16>(&self.board, *make_move));
                *see = Some(see_score);
                if see_score < 0 {
                    continue;
                }
                max = *score;
                best_index = Some(index);
            }
        }
        if let Some(index) = best_index {
            Some(self.queue.swap_remove(index).0)
        } else {
            None
        }
    }
}
