use arrayvec::ArrayVec;
use chess::{ChessMove, Piece, EMPTY};

use crate::bm::bm_eval::eval::Depth::Next;
use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_runner::ab_runner::{LocalContext, SharedContext, SEARCH_PARAMS};
use crate::bm::bm_search::move_entry::MoveEntry;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::EntryType::{Exact, LowerBound, UpperBound};
use crate::bm::bm_util::t_table::{Analysis, EntryType};

use super::move_gen::OrderedMoveGen;
use super::move_gen::QuiescenceSearchMoveGen;

pub trait SearchType {
    const DO_NULL_MOVE: bool;
    const IS_PV: bool;
    const IS_ZW: bool;
    type ZeroWindow: SearchType;
}

pub struct Pv;
pub struct Zw;
pub struct NullMove;

impl SearchType for Pv {
    const DO_NULL_MOVE: bool = false;
    const IS_PV: bool = true;
    const IS_ZW: bool = false;
    type ZeroWindow = Zw;
}

impl SearchType for Zw {
    const DO_NULL_MOVE: bool = true;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type ZeroWindow = Zw;
}

impl SearchType for NullMove {
    const DO_NULL_MOVE: bool = false;
    const IS_PV: bool = false;
    const IS_ZW: bool = true;
    type ZeroWindow = NullMove;
}

const MIN_PIECE_CNT: u32 = 2;

pub fn search<Search: SearchType>(
    position: &mut Position,
    local_context: &mut LocalContext,
    shared_context: &SharedContext,
    ply: u32,
    target_ply: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> (Option<ChessMove>, Evaluation) {
    let depth = target_ply.saturating_sub(ply);
    if ply != 0 && shared_context.abort_absolute(depth, *local_context.nodes()) {
        return (None, Evaluation::max());
    }

    if ply != 0 && position.forced_draw() {
        *local_context.nodes() += 1;
        return (None, Evaluation::new(0));
    }

    local_context.update_sel_depth(ply);
    if ply >= target_ply {
        return (
            None,
            q_search(
                position,
                local_context,
                shared_context,
                0,
                SEARCH_PARAMS.get_q_search_depth(),
                alpha,
                beta,
            ),
        );
    }
    let skip_move = local_context.get_skip_move(ply);
    let tt_entry = if skip_move.is_some() {
        None
    } else {
        shared_context.get_t_table().get(position.board())
    };

    *local_context.nodes() += 1;

    let mut best_move = None;

    let board = *position.board();

    let initial_alpha = alpha;

    let depth = target_ply - ply;

    if let Some(entry) = tt_entry {
        *local_context.tt_hits() += 1;
        best_move = Some(entry.table_move());
        if !Search::IS_PV && ply + entry.depth() >= target_ply {
            let score = entry.score();
            match entry.entry_type() {
                Exact => {
                    return (best_move, score);
                }
                LowerBound => {
                    if score >= beta {
                        return (best_move, score);
                    }
                }
                UpperBound => {
                    if score <= alpha {
                        return (best_move, score);
                    }
                }
            }
        }
    } else {
        *local_context.tt_misses() += 1;
    }

    let in_check = *position.board().checkers() != EMPTY;

    let eval = if skip_move.is_none() {
        position.get_eval()
    } else {
        local_context.get_last_eval(ply + 2).unwrap()
    };

    local_context.push_eval(eval, ply);
    let improving = if let Some(prev_eval) = local_context.get_last_eval(ply) {
        !in_check && eval > prev_eval
    } else {
        false
    };

    let only_pawns =
        MIN_PIECE_CNT + board.pieces(Piece::Pawn).popcnt() >= board.combined().popcnt();
    let do_null_move = !Search::IS_PV
        && SEARCH_PARAMS.do_nmp(depth)
        && !in_check
        && Search::DO_NULL_MOVE
        && !only_pawns;

    if do_null_move && skip_move.is_none() && position.null_move() {
        {
            let threat_table = local_context.get_threat_table();
            while threat_table.len() <= ply as usize + 1 {
                threat_table.push(MoveEntry::new());
            }
        }

        let zw = beta >> Next;
        let reduction = SEARCH_PARAMS.get_nmp().reduction(depth);
        let r_target_ply = target_ply.saturating_sub(reduction);
        let (threat_move, search_score) = search::<NullMove>(
            position,
            local_context,
            shared_context,
            ply + 1,
            r_target_ply,
            zw,
            zw + 1,
        );
        if let Some(threat_move) = threat_move {
            let threat_table = local_context.get_threat_table();
            threat_table[ply as usize + 1].push(threat_move)
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
            local_context,
            shared_context,
            ply,
            target_ply,
            alpha,
            beta,
        );
        best_move = iid_move;
    }

    let do_rev_f_prune =
        !Search::IS_PV && SEARCH_PARAMS.do_rev_fp() && SEARCH_PARAMS.do_rev_f_prune(depth);
    let do_f_prune = !Search::IS_PV && SEARCH_PARAMS.do_fp();

    if !in_check && skip_move.is_none() && do_rev_f_prune {
        let f_margin = SEARCH_PARAMS.get_rev_fp().threshold(depth);
        if eval - f_margin >= beta {
            return (None, eval);
        }
    }

    //This guarantees that threat_table.len() <= ply as usize + 1
    if skip_move.is_none() {
        while local_context.get_k_table().len() <= ply as usize {
            local_context.get_k_table().push(MoveEntry::new());
            local_context.get_threat_table().push(MoveEntry::new());
        }

        if let Some(entry) = local_context.get_k_table().get_mut(ply as usize + 2) {
            entry.clear();
        }
        if let Some(entry) = local_context.get_threat_table().get_mut(ply as usize + 2) {
            entry.clear();
        }
    }

    let mut highest_score = None;

    let threat_move_entry = if ply > 1 {
        local_context.get_threat_table()[ply as usize]
    } else {
        MoveEntry::new()
    };

    let mut move_gen = OrderedMoveGen::new(
        position.board(),
        best_move,
        threat_move_entry.into_iter(),
        local_context.get_k_table()[ply as usize].into_iter(),
    );

    let mut moves_seen = 0;
    let mut move_exists = false;

    let mut quiets = ArrayVec::<ChessMove, 64>::new();

    while let Some(make_move) = move_gen.next(local_context.get_h_table().borrow()) {
        if Some(make_move) == skip_move {
            continue;
        }
        move_exists = true;
        let is_capture = board.piece_on(make_move.get_dest()).is_some();
        let is_promotion = make_move.get_promotion().is_some();
        position.make_move(make_move);
        let gives_check = *position.board().checkers() != EMPTY;
        let is_quiet = !in_check && !is_capture && !is_promotion;

        let mut extension = 0;

        if gives_check {
            extension = 1;
        }

        let mut score;
        if moves_seen == 0 {
            if let Some(entry) = tt_entry {
                if entry.table_move() == make_move
                    && ply != 0
                    && depth >= 7
                    && !entry.score().is_mate()
                    && entry.depth() >= depth - 2
                    && (entry.entry_type() == EntryType::LowerBound
                        || entry.entry_type() == EntryType::Exact)
                {
                    let reduced_plies = ply + depth / 2 - 1;
                    let s_beta = entry.score() - depth as i16 * 3;
                    position.unmake_move();
                    local_context.set_skip_move(ply, make_move);
                    let (_, s_score) = search::<Search::ZeroWindow>(
                        position,
                        local_context,
                        shared_context,
                        ply,
                        reduced_plies,
                        s_beta - 1,
                        s_beta,
                    );
                    local_context.reset_skip_move(ply);
                    if s_score < s_beta {
                        extension += 1;
                    } else if s_beta >= beta {
                        return (Some(make_move), s_beta);
                    }
                    position.make_move(make_move);
                }
            }
            let (_, search_score) = search::<Search>(
                position,
                local_context,
                shared_context,
                ply + 1,
                target_ply + extension,
                beta >> Next,
                alpha >> Next,
            );
            score = search_score << Next;
        } else {
            if SEARCH_PARAMS.do_lmp()
                && is_quiet
                && quiets.len()
                    >= shared_context
                        .get_lmp_lookup()
                        .get(depth as usize, improving as usize)
            {
                position.unmake_move();
                continue;
            }

            let do_fp = !Search::IS_PV && is_quiet && do_f_prune && depth == 1;

            if do_fp && eval + SEARCH_PARAMS.get_fp() < alpha {
                position.unmake_move();
                continue;
            }

            let mut reduction = 0;
            let do_lmr = SEARCH_PARAMS.do_lmr(depth);

            if do_lmr {
                reduction = shared_context
                    .get_lmr_lookup()
                    .get(depth as usize, moves_seen);

                if Search::IS_PV {
                    reduction = reduction.saturating_sub(SEARCH_PARAMS.get_lmr_pv())
                };
                if improving {
                    reduction = reduction.saturating_sub(1);
                }
                if !is_quiet {
                    reduction = reduction.saturating_sub(1);
                }
            }

            let lmr_ply = target_ply.saturating_sub(reduction);
            //Reduced Search/Zero Window if no reduction
            let zw = alpha >> Next;

            let (_, lmr_score) = search::<Search::ZeroWindow>(
                position,
                local_context,
                shared_context,
                ply + 1,
                lmr_ply + extension,
                zw - 1,
                zw,
            );
            score = lmr_score << Next;

            //Do Zero Window Search in case reduction wasn't zero
            if !Search::IS_ZW && reduction > 0 && score > alpha {
                let (_, zw_score) = search::<Search::ZeroWindow>(
                    position,
                    local_context,
                    shared_context,
                    ply + 1,
                    target_ply + extension,
                    zw - 1,
                    zw,
                );
                score = zw_score << Next;
            }
            //All our attempts at reducing has failed
            if score > alpha {
                let (_, search_score) = search::<Search>(
                    position,
                    local_context,
                    shared_context,
                    ply + 1,
                    target_ply + extension,
                    beta >> Next,
                    alpha >> Next,
                );
                score = search_score << Next;
            }
        }

        position.unmake_move();
        moves_seen += 1;

        if highest_score.is_none() || score > highest_score.unwrap() {
            highest_score = Some(score);
            best_move = Some(make_move);
        }
        if score > alpha {
            if score >= beta {
                if skip_move.is_none() {
                    if !is_capture {
                        let killer_table = local_context.get_k_table();
                        killer_table[ply as usize].push(make_move);
                        local_context
                            .get_h_table()
                            .borrow_mut()
                            .cutoff(&board, make_move, &quiets, depth);
                    }

                    let analysis = Analysis::new(depth, LowerBound, score, make_move);
                    shared_context.get_t_table().set(position.board(), analysis);
                }
                return (Some(make_move), score);
            }
            alpha = score;
        }
        if !is_capture && !quiets.is_full() {
            quiets.push(make_move);
        }
    }
    if !move_exists {
        return if *board.checkers() == EMPTY {
            (None, Evaluation::new(0))
        } else {
            (None, Evaluation::new_checkmate(-1))
        };
    }
    let highest_score = highest_score.unwrap();

    if skip_move.is_none() {
        if let Some(final_move) = &best_move {
            let entry_type = if highest_score > initial_alpha {
                Exact
            } else {
                UpperBound
            };

            let analysis = Analysis::new(depth, entry_type, highest_score, *final_move);
            shared_context.get_t_table().set(position.board(), analysis);
        }
    }
    (best_move, highest_score)
}

pub fn q_search(
    position: &mut Position,
    local_context: &mut LocalContext,
    shared_context: &SharedContext,
    ply: u32,
    target_ply: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> Evaluation {
    *local_context.nodes() += 1;

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
                local_context,
                shared_context,
                ply + 1,
                target_ply,
                beta >> Next,
                alpha >> Next,
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
