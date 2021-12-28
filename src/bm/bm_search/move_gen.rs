use cozy_chess::{Board, Move, Piece, PieceMoves};

use crate::bm::bm_eval::evaluator::StdEvaluator;

use crate::bm::bm_util::h_table::{DoubleMoveHistory, HistoryTable};
use arrayvec::ArrayVec;

use super::move_entry::MoveEntryIterator;

const MAX_MOVES: usize = 218;
const LOSING_CAPTURE: i16 = -(2_i16.pow(10));

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

pub struct OrderedMoveGen<const T: usize, const K: usize> {
    move_list: ArrayVec<PieceMoves, 18>,
    pv_move: Option<Move>,
    threat_move_entry: MoveEntryIterator<T>,
    killer_entry: MoveEntryIterator<K>,
    counter_move: Option<Move>,
    prev_move: Option<Move>,
    gen_type: GenType,
    board: Board,

    queue: ArrayVec<(Move, i16), MAX_MOVES>,
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
        let mut move_list = ArrayVec::new();
        board.generate_moves(|piece_moves| {
            move_list.push(piece_moves);
            false
        });
        Self {
            gen_type: GenType::PvMove,
            move_list,
            counter_move,
            prev_move,
            pv_move,
            threat_move_entry,
            killer_entry,
            board: board.clone(),
            queue: ArrayVec::new(),
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
                for &piece_moves in &self.move_list {
                    if piece_moves.from != pv_move.from {
                        continue;
                    }
                    for mv in piece_moves {
                        if mv == pv_move {
                            return Some(pv_move);
                        }
                    }
                }
                self.pv_move = None;
            }
        }
        if self.gen_type == GenType::CalcCaptures {
            for &piece_moves in &self.move_list {
                let mut piece_moves = piece_moves;
                piece_moves.to &= self.board.colors(!self.board.side_to_move());
                for make_move in piece_moves {
                    if Some(make_move) == self.pv_move {
                        continue;
                    }
                    let mut expected_gain =
                        c_hist.get(self.board.side_to_move(), piece_moves.piece, make_move.to);
                    if StdEvaluator::see::<16>(self.board.clone(), make_move) < 0 {
                        expected_gain += LOSING_CAPTURE;
                    }
                    self.queue.push((make_move, expected_gain));
                }
            }

            self.gen_type = GenType::Captures;
        }
        if self.gen_type == GenType::Captures {
            let mut max = LOSING_CAPTURE;
            let mut best_index = None;
            for (index, &(_, score)) in self.queue.iter().enumerate() {
                if score >= max {
                    max = score;
                    best_index = Some(index);
                }
            }
            if let Some(index) = best_index {
                return Some(self.queue.swap_remove(index).0);
            } else {
                self.gen_type = GenType::GenQuiet;
            }
        }
        if self.gen_type == GenType::GenQuiet {
            for &piece_moves in &self.move_list {
                let mut piece_moves = piece_moves;
                piece_moves.to &= !self.board.colors(!self.board.side_to_move());
                for make_move in piece_moves {
                    if Some(make_move) == self.pv_move {
                        continue;
                    }
                    if let Some(piece) = make_move.promotion {
                        match piece {
                            cozy_chess::Piece::Queen => {
                                self.queue.push((make_move, i16::MAX));
                            }
                            _ => {
                                self.queue.push((make_move, i16::MIN));
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

                    self.queue.push((make_move, score));
                }
            }
            self.gen_type = GenType::Killer;
        }
        //Assumes Killer Moves won't repeat
        if self.gen_type == GenType::Killer {
            for make_move in self.killer_entry.clone() {
                let position = self
                    .queue
                    .iter()
                    .position(|(cmp_move, _)| make_move == *cmp_move);
                if let Some(position) = position {
                    self.queue.swap_remove(position);
                    return Some(make_move);
                }
            }
            self.gen_type = GenType::CounterMove;
        }
        if self.gen_type == GenType::CounterMove {
            self.gen_type = GenType::Quiet;
            if let Some(counter_move) = self.counter_move {
                let position = self
                    .queue
                    .iter()
                    .position(|(cmp_move, _)| counter_move == *cmp_move);
                if let Some(position) = position {
                    self.queue.swap_remove(position);
                    return Some(counter_move);
                }
            }
        }
        if self.gen_type == GenType::ThreatMove {
            for make_move in &mut self.threat_move_entry {
                let position = self
                    .queue
                    .iter()
                    .position(|(cmp_move, _)| make_move == *cmp_move);
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
            for (index, &(_, score)) in self.queue.iter().enumerate() {
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

pub struct QuiescenceSearchMoveGen<const SEE_PRUNE: bool> {
    board: Board,
    gen_type: QSearchGenType,
    queue: ArrayVec<(Move, i16), MAX_MOVES>,
}

impl<const SEE_PRUNE: bool> QuiescenceSearchMoveGen<SEE_PRUNE> {
    pub fn new(board: &Board) -> Self {
        Self {
            board: board.clone(),
            gen_type: QSearchGenType::CalcCaptures,
            queue: ArrayVec::new(),
        }
    }
}

impl<const SEE_PRUNE: bool> Iterator for QuiescenceSearchMoveGen<SEE_PRUNE> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gen_type == QSearchGenType::CalcCaptures {
            self.board.generate_moves(|mut piece_moves| {
                piece_moves.to &= self.board.colors(!self.board.side_to_move());
                for make_move in piece_moves {
                    let expected_gain = StdEvaluator::see::<16>(self.board.clone(), make_move);
                    if !SEE_PRUNE || expected_gain > -1 {
                        let pos = self
                            .queue
                            .binary_search_by_key(&expected_gain, |(_, score)| *score)
                            .unwrap_or_else(|pos| pos);
                        self.queue.insert(pos, (make_move, expected_gain));
                    }
                }
                false
            });
            self.gen_type = QSearchGenType::Captures;
        }
        if let Some((make_move, _)) = self.queue.pop() {
            Some(make_move)
        } else {
            None
        }
    }
}
