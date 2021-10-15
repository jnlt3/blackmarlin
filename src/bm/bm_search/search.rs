use arrayvec::ArrayVec;
use chess::{ChessMove, Color, MoveGen, Piece, ALL_COLORS, ALL_PIECES, EMPTY};

use crate::bm::bm_eval::eval::Depth::Next;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_runner::ab_runner::{SearchOptions, SEARCH_PARAMS};
use crate::bm::bm_search::move_entry::MoveEntry;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::Analysis;
use crate::bm::bm_util::t_table::Score::{Exact, LowerBound, UpperBound};

use super::move_gen::OrderedMoveGen;
use super::move_gen::QuiescenceSearchMoveGen;

pub trait SearchType {
    const DO_NULL_MOVE: bool;
    const DO_SINGULAR: bool;
    const IS_PV: bool;
    const IS_ZW: bool;
    type ZeroWindow: SearchType;
}

pub struct Pv;
pub struct Zw;
pub struct NullMove;
pub struct Singular;

impl SearchType for Pv {
    const DO_NULL_MOVE: bool = false;
    const DO_SINGULAR: bool = true;
    const IS_PV: bool = true;
    const IS_ZW: bool = false;
    type ZeroWindow = Zw;
}

impl SearchType for Zw {
    const DO_NULL_MOVE: bool = true;
    const DO_SINGULAR: bool = true;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type ZeroWindow = Zw;
}

impl SearchType for NullMove {
    const DO_NULL_MOVE: bool = false;
    const DO_SINGULAR: bool = true;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type ZeroWindow = NullMove;
}

impl SearchType for Singular {
    const DO_NULL_MOVE: bool = false;
    const DO_SINGULAR: bool = false;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type ZeroWindow = NullMove;
}

const MIN_PIECE_CNT: u32 = 2;

pub fn search<Search: SearchType>(
    position: &mut Position,
    search_options: &mut SearchOptions,
    ply: u32,
    mut target_ply: u32,
    mut alpha: Evaluation,
    mut beta: Evaluation,
    nodes: &mut u32,
) -> (Option<ChessMove>, Evaluation) {
    if ply != 0 && search_options.abort() {
        return (None, Evaluation::new(0));
    }

    if ply != 0 && position.three_fold_repetition() {
        *nodes += 1;
        return (None, Evaluation::new(0));
    }

    search_options.update_sel_depth(ply);
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

    let eval = position.get_eval();

    search_options.push_eval(eval, ply);
    let improving = if let Some(prev_eval) = search_options.get_last_eval(ply) {
        !in_check && eval > prev_eval
    } else {
        false
    };

    let only_pawns =
        MIN_PIECE_CNT + board.pieces(Piece::Pawn).popcnt() >= board.combined().popcnt();
    let do_null_move = !Search::IS_PV
        && SEARCH_PARAMS.do_nmp()
        && !in_check
        && Search::DO_NULL_MOVE
        && !only_pawns;

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
        let (threat_move, search_score) = search::<NullMove>(
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
        let (iid_move, _) = search::<Search>(
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
            return (None, eval);
        }
    }

    //This guarantees that threat_table.len() <= ply as usize + 1
    while search_options.get_k_table().len() <= ply as usize {
        search_options.get_threat_table().push(MoveEntry::new());
        search_options.get_k_table().push(MoveEntry::new());
    }

    let mut highest_score = None;

    let threat_move_entry = if ply > 1 {
        search_options.get_threat_table()[ply as usize]
    } else {
        MoveEntry::new()
    };

    let move_gen = OrderedMoveGen::new(
        position.board(),
        best_move,
        threat_move_entry.into_iter(),
        search_options.get_k_table()[ply as usize].into_iter(),
        search_options,
    );

    let mut moves_seen = 0;
    let mut move_exists = false;

    let mut quiets = ArrayVec::<ChessMove, 64>::new();

    for make_move in move_gen {
        move_exists = true;
        let is_capture = board.piece_on(make_move.get_dest()).is_some();
        let gives_check = *position.board().checkers() != EMPTY;
        let is_promotion = make_move.get_promotion().is_some();
        let is_quiet = !in_check && !gives_check && !is_capture && !is_promotion;

        
        let target_ply = if gives_check {
            target_ply + 1
        } else {
            target_ply
        };
        
        let mut score;
        if moves_seen == 0 {
            moves_seen += 1;
            position.make_move(make_move);
            let (_, search_score) = search::<Search>(
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
            if SEARCH_PARAMS.do_lmp()
                && is_quiet
                && moves_seen
                    >= search_options
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
                    reduction = reduction.saturating_sub(1);
                }
            }

            let lmr_ply = target_ply.saturating_sub(reduction);
            //Reduced Search/Zero Window if no reduction
            let zw = alpha >> Next;

            let (_, lmr_score) = search::<Search::ZeroWindow>(
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
                let (_, zw_score) = search::<Search::ZeroWindow>(
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
                let (_, search_score) = search::<Search>(
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
            if score >= beta {
                if ply != 0 && search_options.abort() {
                    return (None, Evaluation::new(0));
                }
                if !is_capture {
                    let killer_table = search_options.get_k_table();
                    killer_table[ply as usize].push(make_move);
                    search_options.get_h_table().cutoff(
                        &board,
                        make_move,
                        &quiets,
                        depth,
                    );
                }

                let analysis = Analysis::new(depth, LowerBound(score), make_move);
                search_options
                    .get_t_table()
                    .set(position.board(), &analysis);
                return (Some(make_move), score);
            }
            alpha = score;
        }
        if !is_capture {
            if !quiets.is_full() {
                quiets.push(make_move);
            }
        }
    }
    if !move_exists {
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
    (best_move, highest_score)
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
        return position.get_eval();
    }
    let board = *position.board();
    let mut highest_score = None;
    let in_check = *board.checkers() != EMPTY;

    if !in_check {
        let stand_pat = position.get_eval();
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

    let move_gen = QuiescenceSearchMoveGen::<{ SEARCH_PARAMS.do_see_prune() }>::new(&board);
    for make_move in move_gen {
        let is_capture = board.piece_on(make_move.get_dest()).is_some();
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

pub fn singular<Search: SearchType>(
    position: &mut Position,
    search_options: &mut SearchOptions,
    ply: u32,
    target_ply: u32,
    r_beta: Evaluation,
    best_move: ChessMove,
    nodes: &mut u32,
) -> Option<Evaluation> {
    while search_options.get_k_table().len() <= ply as usize {
        search_options.get_threat_table().push(MoveEntry::new());
        search_options.get_k_table().push(MoveEntry::new());
    }

    let threat_move_entry = if ply > 1 {
        search_options.get_threat_table()[ply as usize]
    } else {
        MoveEntry::new()
    };
    let move_gen = OrderedMoveGen::new(
        position.board(),
        Some(best_move),
        threat_move_entry.into_iter(),
        search_options.get_k_table()[ply as usize].into_iter(),
        search_options,
    );

    let mut best_eval = None;

    let r_beta = r_beta >> Next;
    for make_move in move_gen {
        if make_move == best_move {
            continue;
        }
        position.make_move(make_move);

        let (_, eval) = search::<Search::ZeroWindow>(
            position,
            search_options,
            ply + 1,
            target_ply,
            r_beta - 1,
            r_beta,
            nodes,
        );
        position.unmake_move();

        let eval = eval << Next;
        if best_eval.is_none() || eval > best_eval.unwrap() {
            best_eval = Some(eval);
        }
        if eval >= r_beta {
            return best_eval;
        }
    }
    best_eval
}
