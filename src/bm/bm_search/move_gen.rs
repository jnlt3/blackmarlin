use chess::{Board, ChessMove, MoveGen, EMPTY};

use crate::bm::bm_runner::ab_runner::SearchOptions;

use crate::bm::bm_util::evaluator::Evaluator;
use crate::bm::bm_util::h_table::HistoryTable;
use arrayvec::ArrayVec;
use std::marker::PhantomData;
use std::sync::Arc;

use super::move_entry::MoveEntryIterator;

const MAX_MOVES: usize = 218;
const MAX_PROMO_MOVES: usize = 14;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GenType {
    PvMove,
    CalcCaptures,
    Captures,
    GenQuiet,
    QPromotions,
    KPromotions,
    ThreatMove,
    Killer,
    Quiet,
}

#[cfg(not(feature = "advanced_move_gen"))]
pub struct PvMoveGen {
    move_gen: MoveGen,
    board: Board,
    pv_move: Option<ChessMove>,
    gen_type: GenType,
}

#[cfg(not(feature = "advanced_move_gen"))]
impl PvMoveGen {
    pub fn new(board: &Board, pv_move: Option<ChessMove>) -> Self {
        Self {
            gen_type: GenType::PvMove,
            move_gen: MoveGen::new_legal(board),
            board: *board,
            pv_move,
        }
    }
}

#[cfg(not(feature = "advanced_move_gen"))]
impl Iterator for PvMoveGen {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gen_type == GenType::PvMove {
            self.gen_type = GenType::Quiet;
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

#[cfg(feature = "advanced_move_gen")]
pub struct OrderedMoveGen<Eval: Evaluator, const T: usize, const K: usize> {
    move_gen: MoveGen,
    pv_move: Option<ChessMove>,
    threat_move_entry: MoveEntryIterator<T>,
    killer_entry: MoveEntryIterator<K>,
    hist: Arc<HistoryTable>,
    gen_type: GenType,
    board: Board,

    queue: ArrayVec<(ChessMove, i16), MAX_MOVES>,
    mask: [bool; MAX_MOVES],
    queen_promo: ArrayVec<ChessMove, MAX_PROMO_MOVES>,
    knight_promo: ArrayVec<ChessMove, MAX_PROMO_MOVES>,

    eval: PhantomData<Eval>,
}

#[cfg(feature = "advanced_move_gen")]
impl<Eval: 'static + Evaluator + Clone + Send, const T: usize, const K: usize>
    OrderedMoveGen<Eval, T, K>
{
    pub fn new(
        board: &Board,
        pv_move: Option<ChessMove>,
        threat_move_entry: MoveEntryIterator<T>,
        killer_entry: MoveEntryIterator<K>,
        options: &SearchOptions<Eval>,
    ) -> Self {
        Self {
            gen_type: GenType::PvMove,
            move_gen: MoveGen::new_legal(board),
            pv_move,
            threat_move_entry,
            killer_entry,
            hist: options.get_h_table().clone(),
            board: *board,
            queue: ArrayVec::new(),
            queen_promo: ArrayVec::new(),
            knight_promo: ArrayVec::new(),
            mask: [true; MAX_MOVES],
            eval: PhantomData::default(),
        }
    }
}

/*
TODO:
Move ordering via learned pairwise ranking
Use positional encoding to represent moves
[from][to][promotion]
*/

#[cfg(feature = "advanced_move_gen")]
impl<Eval: Evaluator, const K: usize, const T: usize> Iterator for OrderedMoveGen<Eval, K, T> {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        match self.gen_type {
            GenType::PvMove => {
                self.gen_type = GenType::CalcCaptures;
                #[cfg(feature = "pv")]
                if let Some(pv_move) = self.pv_move {
                    if self.board.legal(pv_move) {
                        self.move_gen.remove_move(pv_move);
                        return Some(pv_move);
                    }
                }
                self.next()
            }
            GenType::CalcCaptures => {
                self.move_gen.set_iterator_mask(*self.board.combined());
                for make_move in &mut self.move_gen {
                    let expected_gain = Eval::see(self.board, make_move);
                    let pos = self
                        .queue
                        .binary_search_by_key(&expected_gain, |(_, score)| *score)
                        .unwrap_or_else(|pos| pos);
                    self.queue.insert(pos, (make_move, expected_gain));
                }
                self.gen_type = GenType::Captures;
                self.next()
            }
            GenType::Captures => {
                if let Some((make_move, _)) = self.queue.pop() {
                    return Some(make_move);
                }
                self.gen_type = GenType::GenQuiet;
                self.next()
            }
            GenType::GenQuiet => {
                self.move_gen.set_iterator_mask(!EMPTY);
                let partition = self.queue.len();
                for make_move in &mut self.move_gen {
                    if Some(make_move) == self.pv_move {
                        continue;
                    }
                    #[cfg(feature = "promo_move_ord")]
                    if let Some(piece) = make_move.get_promotion() {
                        match piece {
                            chess::Piece::Queen | chess::Piece::Knight => {}
                            _ => {
                                self.queue.insert(partition, (make_move, i16::MIN));
                                continue;
                            }
                        };
                    }
                    let mut score = 0;
                    let piece = self.board.piece_on(make_move.get_source()).unwrap();
                    #[cfg(feature = "hist")]
                    {
                        score += self
                            .hist
                            .get(self.board.side_to_move(), piece, make_move.get_dest())
                            .min(i16::MAX as u32) as i16;
                    }
                    let pos = self.queue[partition..]
                        .binary_search_by_key(&score, |(_, score)| *score)
                        .unwrap_or_else(|pos| pos);
                    self.queue.insert(partition + pos, (make_move, score));
                }
                self.gen_type = GenType::QPromotions;
                self.next()
            }
            GenType::QPromotions => {
                if let Some(make_move) = self.queen_promo.pop() {
                    return Some(make_move);
                }
                self.gen_type = GenType::KPromotions;
                self.next()
            }
            GenType::KPromotions => {
                if let Some(make_move) = self.knight_promo.pop() {
                    return Some(make_move);
                }
                self.gen_type = GenType::Killer;
                self.next()
            }
            //Assumes Killer Moves won't repeat
            GenType::Killer => {
                #[cfg(feature = "killer")]
                for make_move in &mut self.killer_entry {
                    if Some(make_move) != self.pv_move {
                        let position = self
                            .queue
                            .iter()
                            .position(|(cmp_move, _)| make_move == *cmp_move);
                        if let Some(position) = position {
                            self.mask[position] = false;
                            return Some(make_move);
                        }
                    }
                }
                self.gen_type = GenType::ThreatMove;
                self.next()
            }
            GenType::ThreatMove => {
                #[cfg(feature = "threat")]
                for make_move in &mut self.threat_move_entry {
                    if Some(make_move) != self.pv_move {
                        let position = self
                            .queue
                            .iter()
                            .position(|(cmp_move, _)| make_move == *cmp_move);
                        if let Some(position) = position {
                            self.mask[position] = false;
                            return Some(make_move);
                        }
                    }
                }
                self.gen_type = GenType::Quiet;
                self.next()
            }
            GenType::Quiet => {
                while let Some((make_move, _)) = self.queue.pop() {
                    if self.mask[self.queue.len()] {
                        return Some(make_move);
                    }
                }
                None
            }
        }
    }
}

#[cfg(feature = "q_search_move_ord")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QSearchGenType {
    CalcCaptures,
    Captures,
    Quiet,
}

#[cfg(feature = "q_search_move_ord")]
pub struct QuiescenceSearchMoveGen<Eval: Evaluator, const SEE_PRUNE: bool> {
    move_gen: MoveGen,
    board: Board,
    gen_type: QSearchGenType,
    queue: ArrayVec<(ChessMove, i16), MAX_MOVES>,

    eval: PhantomData<Eval>,
}

#[cfg(feature = "q_search_move_ord")]
impl<Eval: Evaluator, const SEE_PRUNE: bool> QuiescenceSearchMoveGen<Eval, SEE_PRUNE> {
    pub fn new(board: &Board) -> Self {
        Self {
            board: *board,
            move_gen: MoveGen::new_legal(board),
            gen_type: QSearchGenType::CalcCaptures,
            queue: ArrayVec::new(),
            eval: Default::default(),
        }
    }
}

#[cfg(feature = "q_search_move_ord")]
impl<Eval: Evaluator, const SEE_PRUNE: bool> Iterator for QuiescenceSearchMoveGen<Eval, SEE_PRUNE> {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gen_type == QSearchGenType::CalcCaptures {
            self.move_gen.set_iterator_mask(*self.board.combined());
            for make_move in &mut self.move_gen {
                let expected_gain = Eval::see(self.board, make_move);
                if !SEE_PRUNE || expected_gain > -1 {
                    let pos = self
                        .queue
                        .binary_search_by_key(&expected_gain, |(_, score)| *score)
                        .unwrap_or_else(|pos| pos);
                    self.queue.insert(pos, (make_move, expected_gain));
                }
            }
            self.gen_type = QSearchGenType::Captures;
        }
        if self.gen_type == QSearchGenType::Captures {
            if let Some((make_move, _)) = self.queue.pop() {
                return Some(make_move);
            }
            self.move_gen.set_iterator_mask(!*self.board.combined());
            self.gen_type = QSearchGenType::Quiet;
        }
        self.move_gen.next()
    }
}
