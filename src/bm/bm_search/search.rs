use chess::{ChessMove, MoveGen, Piece, EMPTY};

use crate::bm::bm_eval::eval::Depth::Next;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_runner::ab_runner::{SearchOptions, SEARCH_PARAMS};
use crate::bm::bm_search::move_entry::MoveEntry;
use crate::bm::bm_search::move_gen::PvMoveGen;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::Analysis;
use crate::bm::bm_util::t_table::Score::{Exact, LowerBound, UpperBound};

use crate::bm::bm_util::c_hist::PieceTo;
use crate::bm::bm_util::evaluator::Evaluator;

use super::move_gen::{OrderedMoveGen, QuiescenceSearchMoveGen};

pub trait SearchType {
    const DO_NULL_MOVE: bool;

    const IS_PV: bool;

    type OffPv: SearchType;
}

pub struct Pv;

pub struct NonPv;

pub struct NullMove;

impl SearchType for Pv {
    const DO_NULL_MOVE: bool = true;
    const IS_PV: bool = true;
    type OffPv = NonPv;
}

impl SearchType for NonPv {
    const DO_NULL_MOVE: bool = true;
    const IS_PV: bool = false;
    type OffPv = NonPv;
}

impl SearchType for NullMove {
    const DO_NULL_MOVE: bool = false;
    const IS_PV: bool = false;
    type OffPv = NullMove;
}

const MIN_PIECE_CNT: u32 = 2;

pub fn search<Search: SearchType, Eval: Evaluator>(
    position: &mut Position,
    search_options: &mut SearchOptions<Eval>,
    ply: u32,
    target_ply: u32,
    mut alpha: Evaluation,
    mut beta: Evaluation,
    nodes: &mut u32,
) -> (Option<ChessMove>, Evaluation) {
    if ply != 0 && search_options.abort() {
        return (None, Evaluation::new(0));
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
        );
    }
    let tt_entry = search_options.get_t_table().get(position);
    *nodes += 1;

    if position.three_fold_repetition() {
        return (None, Evaluation::new(0));
    }

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
                    return (best_move, score);
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
                return (best_move, entry.score().value());
            }
        }
    } else {
        *search_options.tt_misses() += 1;
    }

    let in_check = *position.board().checkers() != EMPTY;

    let do_null_move = !Search::IS_PV
        && SEARCH_PARAMS.do_nmp()
        && !in_check
        && Search::DO_NULL_MOVE
        && (MIN_PIECE_CNT + board.pieces(Piece::Pawn).popcnt() < board.combined().popcnt());

    if do_null_move && position.null_move() {
        {
            let _count = SEARCH_PARAMS.get_threat_move_cnt();
            let threat_table = search_options.get_threat_table();
            while threat_table.len() <= ply as usize + 1 {
                threat_table.push(MoveEntry::new());
            }
        }

        let zw = beta >> Next;
        let reduction = SEARCH_PARAMS.get_nmp().reduction(depth);
        let r_target_ply = target_ply.max(reduction) - reduction;
        let (threat_move, search_score) = search::<NullMove, Eval>(
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
            return (None, score);
        }
    }

    let do_iid = SEARCH_PARAMS.do_iid(depth) && Search::IS_PV && !in_check;
    if do_iid && best_move.is_none() {
        let reduction = SEARCH_PARAMS.get_iid().reduction(depth);
        let target_ply = target_ply.max(reduction) - reduction;
        let (iid_move, _) = search::<Pv, Eval>(
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

    let do_f_prune =
        !Search::IS_PV && !in_check && SEARCH_PARAMS.do_fp() && SEARCH_PARAMS.do_f_prune(depth);

    let eval = if do_f_prune {
        Some(search_options.eval().evaluate(position))
    } else {
        None
    };

    if !in_check && do_f_prune {
        let f_margin = SEARCH_PARAMS.get_fp().threshold(depth);
        if eval.unwrap() - f_margin >= beta {
            return (None, eval.unwrap() - f_margin);
        }
    }

    {
        let _count = SEARCH_PARAMS.get_k_move_cnt();
        let killer_table = search_options.get_k_table();
        while killer_table.len() <= ply as usize {
            killer_table.push(MoveEntry::new());
        }
        let _count = SEARCH_PARAMS.get_threat_move_cnt();
        let threat_table = search_options.get_threat_table();
        while threat_table.len() <= ply as usize + 1 {
            threat_table.push(MoveEntry::new());
        }
    }

    let mut highest_score = None;

    let threat_move_entry = if ply > 1 {
        search_options.get_threat_table()[ply as usize]
    } else {
        MoveEntry::new()
    };

    let mut piece_to = None;
    if let Some(prev_move) = position.prev_move() {
        piece_to = Some(PieceTo {
            piece: board.piece_on(prev_move.get_dest()).unwrap(),
            to: prev_move.get_dest(),
        });
    }
    let move_gen;
    #[cfg(feature = "advanced_move_gen")]
    {
        move_gen = OrderedMoveGen::new(
            position.board(),
            best_move,
            threat_move_entry.into_iter(),
            search_options.get_k_table()[ply as usize].into_iter(),
            search_options,
            piece_to,
        );
    }

    #[cfg(not(feature = "advanced_move_gen"))]
    {
        move_gen = PvMoveGen::new(position.board(), best_move);
    }

    let mut index = 0;
    *search_options.l2() += 1;

    for make_move in move_gen {
        index += 1;

        let is_capture = board.piece_on(make_move.get_dest()).is_some();

        let gives_check = *position.board().checkers() != EMPTY;
        let is_promotion = make_move.get_promotion().is_some();

        let is_quiet = !in_check && !gives_check && !is_capture && !is_promotion;
        let mut score;
        if index == 1 {
            position.make_move(make_move);
            let (_, search_score) = search::<Search, Eval>(
                position,
                search_options,
                ply + 1,
                target_ply,
                beta >> Next,
                alpha >> Next,
                nodes,
            );
            score = search_score << Next;
        } else {
            if SEARCH_PARAMS.do_lmp(depth)
                && index > search_options.get_lmp_lookup().get(depth as usize)
            {
                break;
            }

            let do_fp = !Search::IS_PV && is_quiet && do_f_prune;

            if do_fp && eval.unwrap() + SEARCH_PARAMS.get_fp().threshold(depth) < alpha {
                continue;
            }
            position.make_move(make_move);

            let mut reduction = 0;
            let do_lmr = SEARCH_PARAMS.do_lmr(depth) && is_quiet;

            if do_lmr {
                let lmr_reduce = search_options
                    .get_lmr_lookup()
                    .get(depth as usize, index - 1);
                reduction = if !Search::IS_PV {
                    lmr_reduce
                } else {
                    lmr_reduce - 1
                };
            }

            let lmr_ply = target_ply.max(reduction) - reduction;
            //Reduced Search/Zero Window if no reduction
            let zw = alpha >> Next;

            let (_, lmr_score) = search::<Search::OffPv, Eval>(
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
            if reduction > 0 && score > alpha {
                let (_, zw_score) = search::<Search::OffPv, Eval>(
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
                let (_, search_score) = search::<Search::OffPv, Eval>(
                    position,
                    search_options,
                    ply + 1,
                    target_ply,
                    beta >> Next,
                    alpha >> Next,
                    nodes,
                );
                score = search_score << Next;
            }
        }
        position.unmake_move();
        if highest_score.is_none() || score > highest_score.unwrap() {
            highest_score = Some(score);
            best_move = Some(make_move);
        }
        if score > alpha {
            let history_table = search_options.get_h_table();
            if score >= beta {
                *search_options.l1() += index;
                if ply != 0 && search_options.abort() {
                    return (None, Evaluation::new(0));
                }
                if !is_capture {
                    let killer_table = search_options.get_k_table();
                    killer_table[ply as usize].push(make_move);
                    let moved_piece = board.piece_on(make_move.get_source()).unwrap();
                    history_table.cutoff(color, moved_piece, make_move.get_dest(), depth * depth);
                    if let Some(piece_to) = piece_to {
                        let c_hist = search_options.get_c_hist();
                        c_hist.add(
                            !color,
                            piece_to.piece,
                            piece_to.to,
                            moved_piece,
                            make_move.get_dest(),
                            depth,
                        );

                        let c_table = search_options.get_c_table();
                        c_table.add(!color, piece_to.piece, piece_to.to, make_move);
                    }
                } else {
                    let ch_table = search_options.get_ch_table();
                    ch_table.add(
                        color,
                        board.piece_on(make_move.get_source()).unwrap(),
                        make_move.get_dest(),
                        board.piece_on(make_move.get_dest()).unwrap(),
                        depth,
                    );
                }
                let analysis = Analysis::new(depth, LowerBound(score), make_move);
                search_options.get_t_table().set(position, &analysis);
                return (Some(make_move), score);
            }
            alpha = score;
        }
    }
    if index == 0 {
        return if *board.checkers() == EMPTY {
            (None, Evaluation::new(0))
        } else {
            (None, Evaluation::new_checkmate(-1))
        };
    }
    if ply != 0 && search_options.abort() {
        return (None, Evaluation::new(0));
    }
    let highest_score = highest_score.unwrap();

    *search_options.l1() += index;
    if let Some(final_move) = &best_move {
        let score = if highest_score > initial_alpha {
            Exact(highest_score)
        } else {
            UpperBound(highest_score)
        };

        let analysis = Analysis::new(depth, score, *final_move);
        search_options.get_t_table().set(position, &analysis);
    }
    (best_move, highest_score)
}

pub fn q_search<Eval: Evaluator + Clone + Send>(
    position: &mut Position,
    search_options: &mut SearchOptions<Eval>,
    ply: u32,
    target_ply: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
    nodes: &mut u32,
) -> Evaluation {
    *nodes += 1;

    if position.three_fold_repetition() {
        return Evaluation::new(0);
    }
    if ply >= target_ply {
        return search_options.eval().evaluate(position);
    }
    let board = *position.board();
    let mut highest_score = None;
    let in_check = *board.checkers() != EMPTY;

    if !in_check {
        let stand_pat = search_options.eval().evaluate(position);
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
    let move_gen = QuiescenceSearchMoveGen::<Eval, { SEARCH_PARAMS.do_see_prune() }>::new(&board);
    for make_move in move_gen {
        let is_capture = board.piece_on(make_move.get_dest()).is_some();

        #[cfg(not(feature = "q_search_move_ord"))]
        {
            let do_see_prune = SEARCH_PARAMS.do_see_prune() && is_capture && !in_check;
            if do_see_prune && Eval::see(board, make_move) < 0 {
                continue;
            }
        }
        position.make_move(make_move);
        let gives_check = *board.checkers() != EMPTY;

        if in_check || gives_check || is_capture {
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
        }
        position.unmake_move();
    }
    highest_score.unwrap_or(alpha)
}
