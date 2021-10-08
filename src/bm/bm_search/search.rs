use std::slice::SliceIndex;

use chess::{ChessMove, Piece, EMPTY};

use crate::bm::bm_eval::eval::Depth::Next;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_eval::evaluator::StdEvaluator;
use crate::bm::bm_runner::ab_runner::{SearchOptions, SEARCH_PARAMS};
use crate::bm::bm_search::move_entry::MoveEntry;
#[cfg(not(feature = "advanced_move_gen"))]
use crate::bm::bm_search::move_gen::PvMoveGen;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::Analysis;
use crate::bm::bm_util::t_table::Score::{Exact, LowerBound, UpperBound};

#[cfg(feature = "advanced_move_gen")]
use super::move_gen::OrderedMoveGen;
#[cfg(feature = "q_search_move_ord")]
use super::move_gen::QuiescenceSearchMoveGen;
#[cfg(not(feature = "q_search_move_ord"))]
use chess::MoveGen;

pub trait SearchType {
    const DO_NULL_MOVE: bool;
    const IS_PV: bool;
    const IS_ZW: bool;
    type OffPv: SearchType;
    type ZeroWindow: SearchType;
}

pub struct Pv;

pub struct NonPv;

pub struct Zw;

pub struct NullMove;

impl SearchType for Pv {
    const DO_NULL_MOVE: bool = false;
    const IS_PV: bool = true;
    const IS_ZW: bool = false;
    type OffPv = NonPv;
    type ZeroWindow = Zw;
}

impl SearchType for NonPv {
    const DO_NULL_MOVE: bool = true;
    const IS_PV: bool = false;
    const IS_ZW: bool = false;
    type OffPv = NonPv;
    type ZeroWindow = Zw;
}

impl SearchType for Zw {
    const DO_NULL_MOVE: bool = true;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type OffPv = NonPv;
    type ZeroWindow = Zw;
}

impl SearchType for NullMove {
    const DO_NULL_MOVE: bool = false;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type OffPv = NullMove;
    type ZeroWindow = NullMove;
}

const MIN_PIECE_CNT: u32 = 2;

pub fn search<Search: SearchType>(
    position: &mut Position,
    search_options: &mut SearchOptions,
    ply: u32,
    target_ply: u32,
    mut alpha: Evaluation,
    mut beta: Evaluation,
    nodes: &mut u32,
) -> (Option<ChessMove>, Evaluation, u32) {
    if ply != 0 && search_options.abort() {
        return (None, Evaluation::new(0), ply);
    }

    if ply != 0 && position.three_fold_repetition() {
        *nodes += 1;
        return (None, Evaluation::new(0), ply);
    }

    if ply >= target_ply {
        return (
            None,
            q_search(
                position,
                search_options,
                0,
                SEARCH_PARAMS.get_q_search_depth(),
                alpha,
                beta,
                nodes,
            ),
            ply,
        );
    }
    let tt_entry = search_options.get_t_table().get(position.board());
    *nodes += 1;

    let mut best_move = None;

    let board = *position.board();
    let color = board.side_to_move();

    let initial_alpha = alpha;

    let depth = target_ply - ply;

    if let Some(entry) = tt_entry {
        *search_options.tt_hits() += 1;
        best_move = Some(entry.table_move());
        if !Search::IS_PV && ply + entry.depth() >= target_ply {
            match entry.score() {
                Exact(score) => {
                    return (best_move, score, ply);
                }
                LowerBound(score) => {
                    if score > alpha {
                        alpha = score;
                    }
                }
                UpperBound(score) => {
                    if score < beta {
                        beta = score;
                    }
                }
            }
            if alpha >= beta {
                return (best_move, entry.score().value(), ply);
            }
        }
    } else {
        *search_options.tt_misses() += 1;
    }

    let in_check = *position.board().checkers() != EMPTY;

    let eval = search_options.eval().evaluate(position.board());
    search_options.push_eval(eval, ply);
    let improving = if let Some(prev_eval) = search_options.get_last_eval(ply) {
        !in_check && eval > prev_eval
    } else {
        false
    };

    let do_null_move = !Search::IS_PV
        && SEARCH_PARAMS.do_nmp()
        && !in_check
        && Search::DO_NULL_MOVE
        && (MIN_PIECE_CNT + board.pieces(Piece::Pawn).popcnt() < board.combined().popcnt());

    if do_null_move && position.null_move() {
        {
            let threat_table = search_options.get_threat_table();
            while threat_table.len() <= ply as usize + 1 {
                threat_table.push(MoveEntry::new());
            }
        }

        let zw = beta >> Next;
        let reduction = SEARCH_PARAMS.get_nmp().reduction(depth);
        let r_target_ply = target_ply.max(reduction) - reduction;
        let (threat_move, search_score, _) = search::<NullMove>(
            position,
            search_options,
            ply + 1,
            r_target_ply,
            zw,
            zw + 1,
            nodes,
        );
        if let Some(threat_move) = threat_move {
            search_options.get_threat_table()[ply as usize + 1].push(threat_move)
        }
        position.unmake_move();
        let score = search_score << Next;
        if score >= beta {
            return (None, score, ply);
        }
    }

    let do_iid = SEARCH_PARAMS.do_iid(depth) && Search::IS_PV && !in_check;
    if do_iid && best_move.is_none() {
        let reduction = SEARCH_PARAMS.get_iid().reduction(depth);
        let target_ply = target_ply.max(reduction) - reduction;
        let (iid_move, _, _) = search::<Search>(
            position,
            search_options,
            ply,
            target_ply,
            alpha,
            beta,
            nodes,
        );
        best_move = iid_move;
    }

    let do_rev_f_prune =
        !Search::IS_PV && SEARCH_PARAMS.do_rev_fp() && SEARCH_PARAMS.do_rev_f_prune(depth);
    let do_f_prune = !Search::IS_PV && SEARCH_PARAMS.do_fp() && SEARCH_PARAMS.do_f_prune(depth);

    if !in_check && do_rev_f_prune {
        let f_margin = SEARCH_PARAMS.get_rev_fp().threshold(depth);
        if eval - f_margin >= beta {
            return (None, eval, ply);
        }
    }
    {
        //This guarantees that threat_table.len() <= ply as usize + 1
        while search_options.get_k_table().len() <= ply as usize {
            search_options.get_threat_table().push(MoveEntry::new());
            search_options.get_k_table().push(MoveEntry::new());
        }
    }

    let mut highest_score = None;

    let threat_move_entry = if ply > 1 {
        search_options.get_threat_table()[ply as usize]
    } else {
        MoveEntry::new()
    };

    let move_gen;
    #[cfg(feature = "advanced_move_gen")]
    {
        move_gen = OrderedMoveGen::new(
            position.board(),
            best_move,
            threat_move_entry.into_iter(),
            search_options.get_k_table()[ply as usize].into_iter(),
            search_options,
        );
    }

    #[cfg(not(feature = "advanced_move_gen"))]
    {
        move_gen = PvMoveGen::new(position.board(), best_move);
    }

    let mut moves_seen = 0;
    let mut move_exists = false;
    let mut sel_depth = 0;
    for make_move in move_gen {
        move_exists = true;
        let is_capture = board.piece_on(make_move.get_dest()).is_some();

        let gives_check = *position.board().checkers() != EMPTY;
        let is_promotion = make_move.get_promotion().is_some();

        let is_quiet = !in_check && !gives_check && !is_capture && !is_promotion;
        let mut score;
        if moves_seen == 0 {
            moves_seen += 1;
            position.make_move(make_move);
            let (_, search_score, d) = search::<Search>(
                position,
                search_options,
                ply + 1,
                target_ply,
                beta >> Next,
                alpha >> Next,
                nodes,
            );
            score = search_score << Next;
            sel_depth = sel_depth.max(d);
        } else {
            if SEARCH_PARAMS.do_lmp(depth)
                && is_quiet
                && moves_seen
                    > search_options
                        .get_lmp_lookup()
                        .get(depth as usize, improving as usize)
            {
                continue;
            }

            let do_fp = !Search::IS_PV && is_quiet && do_f_prune;

            if do_fp && eval + SEARCH_PARAMS.get_fp().threshold(depth) < alpha {
                continue;
            }
            position.make_move(make_move);

            moves_seen += 1;

            let mut reduction = 0;
            let do_lmr = SEARCH_PARAMS.do_lmr(depth) && is_quiet;

            if do_lmr {
                let lmr_reduce = search_options
                    .get_lmr_lookup()
                    .get(depth as usize, moves_seen);
                reduction = if !Search::IS_PV {
                    lmr_reduce
                } else {
                    lmr_reduce.saturating_sub(SEARCH_PARAMS.get_lmr_pv())
                };
                if improving {
                    reduction = reduction.saturating_sub(1)
                }
            }

            let lmr_ply = target_ply.saturating_sub(reduction);
            //Reduced Search/Zero Window if no reduction
            let zw = alpha >> Next;

            let (_, lmr_score, _) = search::<Search::ZeroWindow>(
                position,
                search_options,
                ply + 1,
                lmr_ply,
                zw - 1,
                zw,
                nodes,
            );
            score = lmr_score << Next;

            //Do Zero Window Search in case reduction wasn't zero
            if !Search::IS_ZW && reduction > 0 && score > alpha {
                let (_, zw_score, _) = search::<Search::ZeroWindow>(
                    position,
                    search_options,
                    ply + 1,
                    target_ply,
                    zw - 1,
                    zw,
                    nodes,
                );
                score = zw_score << Next;
            }
            //All our attempts at reducing has failed
            if score > alpha {
                let (_, search_score, d) = search::<Search::OffPv>(
                    position,
                    search_options,
                    ply + 1,
                    target_ply,
                    beta >> Next,
                    alpha >> Next,
                    nodes,
                );
                score = search_score << Next;
                sel_depth = sel_depth.max(d);
            }
        }
        position.unmake_move();
        if highest_score.is_none() || score > highest_score.unwrap() {
            highest_score = Some(score);
            best_move = Some(make_move);
        }
        if score > alpha {
            if score >= beta {
                if ply != 0 && search_options.abort() {
                    return (None, Evaluation::new(0), ply);
                }
                if !is_capture {
                    let killer_table = search_options.get_k_table();
                    killer_table[ply as usize].push(make_move);
                    let moved_piece = board.piece_on(make_move.get_source()).unwrap();
                    search_options.get_h_table().cutoff(
                        color,
                        moved_piece,
                        make_move.get_dest(),
                        depth * depth,
                    );
                }

                let analysis = Analysis::new(depth, LowerBound(score), make_move);
                search_options
                    .get_t_table()
                    .set(position.board(), &analysis);
                return (Some(make_move), score, sel_depth);
            }
            alpha = score;
        }
    }
    if !move_exists {
        return if *board.checkers() == EMPTY {
            (None, Evaluation::new(0), ply)
        } else {
            (None, Evaluation::new_checkmate(-1), ply)
        };
    }
    if ply != 0 && search_options.abort() {
        return (None, Evaluation::new(0), ply);
    }
    let highest_score = highest_score.unwrap();

    if let Some(final_move) = &best_move {
        let score = if highest_score > initial_alpha {
            Exact(highest_score)
        } else {
            UpperBound(highest_score)
        };

        let analysis = Analysis::new(depth, score, *final_move);
        search_options
            .get_t_table()
            .set(position.board(), &analysis);
    }
    (best_move, highest_score, sel_depth)
}

pub fn q_search(
    position: &mut Position,
    search_options: &mut SearchOptions,
    ply: u32,
    target_ply: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
    nodes: &mut u32,
) -> Evaluation {
    *nodes += 1;
    if ply >= target_ply {
        return search_options.eval().evaluate(position.board());
    }
    let board = *position.board();
    let mut highest_score = None;
    let in_check = *board.checkers() != EMPTY;

    if !in_check {
        let stand_pat = search_options.eval().evaluate(position.board());
        let do_dp = SEARCH_PARAMS.do_dp();
        if do_dp && stand_pat + SEARCH_PARAMS.get_delta() < alpha {
            return stand_pat;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
            highest_score = Some(stand_pat);
            if stand_pat >= beta {
                return stand_pat;
            }
        }
    }

    #[cfg(not(feature = "q_search_move_ord"))]
    let move_gen = MoveGen::new_legal(&board);
    #[cfg(feature = "q_search_move_ord")]
    let move_gen = QuiescenceSearchMoveGen::<{ SEARCH_PARAMS.do_see_prune() }>::new(&board);
    for make_move in move_gen {
        let is_capture = board.piece_on(make_move.get_dest()).is_some();

        #[cfg(not(feature = "q_search_move_ord"))]
        {
            let do_see_prune = SEARCH_PARAMS.do_see_prune() && is_capture && !in_check;
            if do_see_prune && StdEvaluator::see(board, make_move) < 0 {
                continue;
            }
        }

        if in_check || is_capture {
            position.make_move(make_move);
            let search_score = q_search(
                position,
                search_options,
                ply + 1,
                target_ply,
                beta >> Next,
                alpha >> Next,
                nodes,
            );
            let score = search_score << Next;
            if highest_score.is_none() || score > highest_score.unwrap() {
                highest_score = Some(score);
            }
            if score > alpha {
                alpha = score;
                if score >= beta {
                    position.unmake_move();
                    return score;
                }
            }
            position.unmake_move();
        }
    }
    highest_score.unwrap_or(alpha)
}
