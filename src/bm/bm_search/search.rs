use arrayvec::ArrayVec;
use cozy_chess::{Board, Move, Piece};

use crate::bm::bm_runner::ab_runner::{MoveData, SharedContext, ThreadContext, MAX_PLY};
use crate::bm::bm_util::eval::Depth::Next;
use crate::bm::bm_util::eval::Evaluation;
use crate::bm::bm_util::history::HistoryIndices;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::Bounds;

use super::move_gen::{OrderedMoveGen, Phase, QSearchMoveGen};
use super::see::compare_see;

pub trait SearchType {
    const NM: bool;
    const PV: bool;
    type Zw: SearchType;
}

pub struct Pv;
pub struct Zw;
pub struct NoNm;

impl SearchType for Pv {
    const NM: bool = false;
    const PV: bool = true;
    type Zw = Zw;
}

impl SearchType for Zw {
    const NM: bool = true;
    const PV: bool = false;
    type Zw = Zw;
}

impl SearchType for NoNm {
    const NM: bool = false;
    const PV: bool = false;
    type Zw = NoNm;
}

const fn do_rev_fp(depth: u32) -> bool {
    depth <= 9
}

const fn rev_fp(depth: u32, improving: bool) -> i16 {
    depth as i16 * 71 - improving as i16 * 62
}

const fn do_razor(depth: u32) -> bool {
    depth <= 4
}

const fn razor_margin(depth: u32) -> i16 {
    depth as i16 * 306
}

const fn razor_qsearch() -> i16 {
    277
}

fn do_nmp<Search: SearchType>(
    board: &Board,
    depth: u32,
    eval: i16,
    beta: i16,
    nstm_threat: bool,
) -> bool {
    Search::NM
        && depth > 4
        && !(nstm_threat && depth <= 7)
        && eval >= beta
        && (board.pieces(Piece::Pawn) | board.pieces(Piece::King)) != board.occupied()
}

fn nmp_depth(depth: u32, eval: i16, beta: i16) -> u32 {
    assert!(eval >= beta);
    let r = 4 + depth * 23 / 60 + ((eval - beta) / 204) as u32;
    depth.saturating_sub(r).max(1)
}

const fn iir(depth: u32) -> u32 {
    if depth >= 4 {
        1
    } else {
        0
    }
}

const fn fp(depth: u32) -> i16 {
    depth as i16 * 86
}

const fn see_fp(depth: u32) -> i16 {
    depth as i16 * 123
}

const fn hp(depth: u32) -> i32 {
    -((depth * depth) as i32) * 138 / 10
}

const fn history_lmr(history: i16) -> i16 {
    history / 112
}

pub fn search<Search: SearchType>(
    pos: &mut Position,
    thread: &mut ThreadContext,
    shared_context: &SharedContext,
    ply: u32,
    mut depth: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
    cut_node: bool,
) -> Evaluation {
    thread.ss[ply as usize].pv_len = 0;

    if ply != 0 && (thread.abort || shared_context.abort_search(thread.nodes())) {
        thread.trigger_abort();
        return Evaluation::min();
    }

    thread.update_sel_depth(ply);
    if ply != 0 && pos.forced_draw(ply) {
        thread.increment_nodes();
        return Evaluation::new(0);
    }

    /*
    At depth 0, we run Quiescence Search
    */
    if depth == 0 || ply >= MAX_PLY {
        return q_search(pos, thread, shared_context, ply, alpha, beta);
    }

    let skip_move = thread.ss[ply as usize].skip_move;
    let mut tt_entry = match skip_move {
        Some(_) => None,
        None => shared_context.get_t_table().get(pos.board()),
    };

    thread.increment_nodes();

    let mut best_move = None;

    let initial_alpha = alpha;

    /*
    Transposition Table
    If we get a TT hit and the search is deep enough,
    we can use the score from TT to cause an early cutoff
    We also use the best move from the transposition table
    to help with move ordering
    */
    if let Some(entry) = tt_entry {
        best_move = entry.table_move.filter(|&mv| pos.board().is_legal(mv));
        thread.tt_hits += 1;
        if entry.table_move.is_some() && best_move.is_none() {
            tt_entry = None;
        }
        if !Search::PV && entry.depth >= depth {
            let score = entry.score;
            match entry.bounds {
                Bounds::Exact => {
                    return score;
                }
                Bounds::LowerBound => {
                    if score >= beta {
                        return score;
                    }
                }
                Bounds::UpperBound => {
                    if score <= alpha {
                        return score;
                    }
                }
            }
        }
    } else {
        thread.tt_misses += 1;
    }

    let in_check = !pos.board().checkers().is_empty();

    let tt_eval = tt_entry.and_then(|entry| entry.eval);
    let raw_eval = match skip_move {
        Some(_) => thread.ss[ply as usize].eval,
        None => tt_eval.unwrap_or_else(|| pos.get_eval()),
    };
    let aggr = pos.aggression(thread.stm, thread.eval);
    let eval = raw_eval + aggr;

    thread.ss[ply as usize].aggr = aggr;
    thread.ss[ply as usize].eval = raw_eval;

    let prev_move_eval = match ply {
        2.. => Some(thread.ss[ply as usize - 2].full_eval()),
        _ => None,
    };
    let improving = match prev_move_eval {
        Some(prev_move_eval) => !in_check && eval > prev_move_eval,
        None => false,
    };

    let (stm_threats, nstm_threats) = pos.threats();
    if !Search::PV && !in_check && skip_move.is_none() {
        /*
        Reverse Futility Pruning:
        If in a non PV node and evaluation is higher than beta + a depth dependent margin
        we assume we can at least achieve beta
        */
        if do_rev_fp(depth) && eval - rev_fp(depth, improving && nstm_threats.is_empty()) >= beta {
            return (eval * 2 + beta) / 3;
        }

        let razor_margin = razor_margin(depth);
        if do_razor(depth) && eval + razor_margin <= alpha {
            let zw = alpha - razor_qsearch();
            let q_search = q_search(pos, thread, shared_context, ply, zw, zw + 1);
            if q_search <= zw {
                return q_search;
            }
        }

        /*
        Null Move Pruning:
        If in a non PV node and we can still achieve beta at a reduced depth after
        giving the opponent the side to move we can prune this node and return the evaluation
        While doing null move pruning, we also get the "best move" for the opponent in case
        This is seen as the major threat in the current position and can be used in
        move ordering for the next ply
        */
        let nmp_eval = tt_entry.map_or(eval, |entry| match entry.bounds {
            Bounds::LowerBound => entry.score.max(eval),
            Bounds::Exact => entry.score,
            Bounds::UpperBound => entry.score.min(eval),
        });
        if do_nmp::<Search>(
            pos.board(),
            depth,
            nmp_eval.raw(),
            beta.raw(),
            !nstm_threats.is_empty(),
        ) && pos.null_move()
        {
            thread.ss[ply as usize].move_played = None;

            let nmp_depth = nmp_depth(depth, nmp_eval.raw(), beta.raw());
            let zw = beta >> Next;
            let search_score = search::<NoNm>(
                pos,
                thread,
                shared_context,
                ply + 1,
                nmp_depth,
                zw,
                zw + 1,
                !cut_node,
            );
            pos.unmake_move();
            let score = search_score << Next;
            if score >= beta {
                let mut verified = depth < 10;
                if !verified {  
                    let verification = search::<NoNm>(
                        pos,
                        thread,
                        shared_context,
                        ply + 1,
                        nmp_depth,
                        alpha,
                        beta,
                        false,
                    );
                    verified = verification >= beta;
                }
                if verified {
                    return score;
                }
            }
        }
    }

    if tt_entry.map_or(true, |entry| entry.depth + 4 < depth) {
        depth -= iir(depth)
    }

    if let Some(entry) = thread.killer_moves.get_mut(ply as usize + 1) {
        entry.clear();
    }

    let mut highest_score = None;

    let prev_move = |prev: u32| match ply >= prev {
        true => thread.ss[(ply - prev) as usize].move_played,
        false => None,
    };

    let cont_1 = prev_move(1);
    let cont_2 = prev_move(2);
    let cont_4 = prev_move(4);

    let killers = thread.killer_moves[ply as usize];
    let mut move_gen = OrderedMoveGen::new(best_move, killers);

    let mut moves_seen = 0;
    let mut move_exists = false;

    let mut quiets = ArrayVec::<Move, 64>::new();
    let mut captures = ArrayVec::<Move, 64>::new();

    let hist_indices = HistoryIndices::new(cont_1, cont_2, cont_4);
    while let Some(make_move) = move_gen.next(pos, &thread.history, &hist_indices) {
        let move_nodes = thread.nodes();
        if Some(make_move) == skip_move {
            continue;
        }

        move_exists = true;
        let is_capture = pos.is_capture(make_move);

        let h_score = match is_capture {
            true => thread.history.get_capture(pos, make_move),
            false => {
                (thread.history.get_quiet(pos, make_move)
                    + thread
                        .history
                        .get_counter_move(pos, &hist_indices, make_move)
                        .unwrap_or_default())
                    / 2
            }
        };
        thread.ss[ply as usize + 1].pv_len = 0;

        let mut extension: i32 = 0;
        let mut score;

        /*
        Singular Extensions:
        If a move can't be beaten by any other move, we assume the move
        is singular (only solution) and extend in order to get a more accurate
        estimation of best move/eval
        */
        if let Some(entry) = tt_entry {
            let multi_cut = depth >= 7;
            if moves_seen == 0
                && entry.table_move == Some(make_move)
                && ply != 0
                && !entry.score.is_mate()
                && entry.depth + 2 >= depth
                && matches!(entry.bounds, Bounds::LowerBound | Bounds::Exact)
                && (multi_cut || eval <= alpha)
            {
                let s_beta = entry.score - depth as i16;
                thread.ss[ply as usize].skip_move = Some(make_move);

                let s_score = match multi_cut {
                    true => search::<Search::Zw>(
                        pos,
                        thread,
                        shared_context,
                        ply,
                        depth / 2 - 1,
                        s_beta - 1,
                        s_beta,
                        cut_node,
                    ),
                    false => eval,
                };

                thread.ss[ply as usize].skip_move = None;
                if s_score < s_beta {
                    extension = 1;
                    if !Search::PV && multi_cut && s_score < s_beta {
                        extension += 1;
                        if !is_capture && s_score + 197 < s_beta {
                            extension += 1;
                        }
                    }
                    if Search::PV && multi_cut && s_score + 120 < s_beta {
                        extension += 1;
                    }
                    if !Search::PV && !multi_cut && eval + 100 <= alpha {
                        extension += 1;
                    }
                    thread.history.update_history(
                        pos,
                        &hist_indices,
                        make_move,
                        &[],
                        &[],
                        depth as i16,
                    );
                } else if multi_cut && s_beta >= beta {
                    /*
                    Multi-cut:
                    If a move isn't singular and the move that disproves the singularity
                    our singular beta is above beta, we assume the move is good enough to beat beta
                    */
                    return s_beta;
                } else if multi_cut && entry.score >= beta {
                    extension = -2;
                } else if multi_cut && cut_node {
                    extension = -2;
                }
            }
        }
        let mut reduction = shared_context
            .get_lmr_lookup()
            .get(depth as usize, moves_seen) as i16;
        reduction -= history_lmr(h_score);

        let lmr_depth = (depth as i16 - reduction).max(1) as u32;

        let non_mate_line = highest_score.map_or(false, |s: Evaluation| !s.is_mate());
        /*
        In non-PV nodes If a move isn't good enough to beat alpha - a static margin
        we assume it's safe to prune this move
        */
        let do_fp = !Search::PV && non_mate_line && moves_seen > 0 && !is_capture && depth <= 9;

        if do_fp && eval + fp(lmr_depth) <= alpha {
            move_gen.skip_quiets();
            continue;
        }

        /*
        If a move is placed late in move ordering, we can safely prune it based on a depth related margin
        */
        if non_mate_line
            && !is_capture
            && quiets.len()
                >= shared_context
                    .get_lmp_lookup()
                    .get(depth as usize, improving as usize)
        {
            move_gen.skip_quiets();
            continue;
        }

        let good_capture = move_gen.phase() <= Phase::GoodCaptures;
        /*
        In low depth, non-PV nodes, we assume it's safe to prune a move
        if it has very low history
        */
        let do_hp = !Search::PV
            && non_mate_line
            && moves_seen > 0
            && depth <= 6
            && (!good_capture || eval <= alpha);

        if do_hp && (h_score as i32) < hp(depth) {
            continue;
        }

        /*
        In non-PV nodes If a move evaluated by SEE isn't good enough to beat alpha - a static margin
        we assume it's safe to prune this move
        */
        let do_see_prune = !Search::PV
            && non_mate_line
            && moves_seen > 0
            && depth <= 6
            && !alpha.is_mate()
            && !good_capture;

        if do_see_prune {
            let see_margin = (alpha - eval - see_fp(depth) + 1).raw();
            if see_margin > 0 || !compare_see(pos.board(), make_move, see_margin) {
                continue;
            }
        }

        thread.ss[ply as usize].move_played = Some(MoveData::from_move(pos.board(), make_move));
        pos.make_move_fetch(make_move, |board| {
            shared_context.get_t_table().prefetch(&board)
        });
        let (_, new_stm_threat) = pos.threats();

        let gives_check = !pos.board().checkers().is_empty();
        if gives_check {
            extension = extension.max(1);
        }

        /*
        LMR
        We try to prove a move is worse than alpha at a reduced depth
        If the move proves to be worse than alpha, we don't have to do a
        full depth search
        */

        if moves_seen > 0 {
            if ply <= (depth + ply) * 2 / 5 {
                reduction -= 1;
            }
            if !Search::PV {
                reduction += 1;
            };
            if !improving {
                reduction += 1;
            }
            if killers.contains(make_move) {
                reduction -= 1;
            }
            if cut_node {
                reduction += 1;
            }
            if new_stm_threat.len() > stm_threats.len() {
                reduction -= 1;
            }
            reduction = reduction.min(depth as i16 - 2).max(0);
        }

        if moves_seen == 0 {
            let depth = (depth as i32 + extension) as u32;
            let search_score = search::<Search>(
                pos,
                thread,
                shared_context,
                ply + 1,
                depth - 1,
                beta >> Next,
                alpha >> Next,
                false,
            );
            score = search_score << Next;
        } else {
            let depth = (depth as i32 + extension) as u32;
            let lmr_depth = (depth as i16 - reduction) as u32;
            //Reduced Search/Zero Window if no reduction
            let zw = alpha >> Next;

            let lmr_score = search::<Search::Zw>(
                pos,
                thread,
                shared_context,
                ply + 1,
                lmr_depth - 1,
                zw - 1,
                zw,
                true,
            );
            score = lmr_score << Next;

            /*
            If no reductions occured in LMR we don't waste time re-searching
            otherwise, we run a full depth search to attempt a fail low
            */
            if lmr_depth < depth && score > alpha {
                let zw_score = search::<Search::Zw>(
                    pos,
                    thread,
                    shared_context,
                    ply + 1,
                    depth - 1,
                    zw - 1,
                    zw,
                    !cut_node,
                );
                score = zw_score << Next;
            }
            /*
            If we don't get a fail low, this means the move has to be searched fully
            */
            if Search::PV && score > alpha {
                let search_score = search::<Search>(
                    pos,
                    thread,
                    shared_context,
                    ply + 1,
                    depth - 1,
                    beta >> Next,
                    alpha >> Next,
                    false,
                );
                score = search_score << Next;
            }
        }

        pos.unmake_move();
        moves_seen += 1;

        if ply == 0 {
            let searched_nodes = thread.nodes() - move_nodes;
            thread.root_nodes[make_move.from as usize][make_move.to as usize] += searched_nodes;
        }

        if highest_score.is_none() || score > highest_score.unwrap() {
            highest_score = Some(score);
            if score > alpha {
                best_move = Some(make_move);
                if (Search::PV || (ply == 0 && moves_seen == 1)) && !thread.abort {
                    let (child_pv, len) = {
                        let child = &thread.ss[ply as usize + 1];
                        (child.pv, child.pv_len)
                    };
                    thread.ss[ply as usize].update_pv(make_move, &child_pv[..len]);
                }
                if score >= beta {
                    if !thread.abort {
                        let amt = depth + (eval <= alpha) as u32 + (score - 50 > beta) as u32;
                        if !is_capture {
                            thread.killer_moves[ply as usize].push(make_move);
                        }
                        thread.history.update_history(
                            pos,
                            &hist_indices,
                            make_move,
                            &quiets,
                            &captures,
                            amt as i16,
                        );
                    }
                    break;
                }
                alpha = score;
            }
        }
        if is_capture {
            if !captures.is_full() {
                captures.push(make_move);
            }
        } else if !quiets.is_full() {
            quiets.push(make_move);
        }
    }
    if !move_exists {
        return match pos.board().checkers().is_empty() {
            true => Evaluation::new(0),
            false => Evaluation::new_checkmate(-1),
        };
    }
    let highest_score = highest_score.unwrap();

    if skip_move.is_none() && !thread.abort {
        let entry_type = match () {
            _ if highest_score <= initial_alpha => Bounds::UpperBound,
            _ if highest_score >= beta => Bounds::LowerBound,
            _ => Bounds::Exact,
        };
        shared_context.get_t_table().set(
            pos.board(),
            depth,
            entry_type,
            highest_score,
            best_move,
            raw_eval,
        );
    }
    highest_score
}

/*
Quiescence Search is a form of search that only searches tactical moves to achieve a quiet position.
This is done as the static evaluation function isn't suited to detecting tactical aspects of the position.
*/

pub fn q_search(
    pos: &mut Position,
    thread: &mut ThreadContext,
    shared_context: &SharedContext,
    ply: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> Evaluation {
    if thread.abort || shared_context.abort_search(thread.nodes()) {
        thread.trigger_abort();
        return Evaluation::min();
    }

    thread.increment_nodes();

    thread.update_sel_depth(ply);
    if ply >= MAX_PLY {
        return pos.get_eval() + pos.aggression(thread.stm, thread.eval);
    }

    let mut best_move = None;
    let initial_alpha = alpha;
    let tt_entry = shared_context.get_t_table().get(pos.board());
    if let Some(entry) = tt_entry {
        best_move = entry.table_move;
        match entry.bounds {
            Bounds::LowerBound => {
                if entry.score >= beta {
                    return entry.score;
                }
            }
            Bounds::Exact => return entry.score,
            Bounds::UpperBound => {
                if entry.score <= alpha {
                    return entry.score;
                }
            }
        }
    }

    let mut highest_score = None;
    let in_check = !pos.board().checkers().is_empty();

    let tt_eval = tt_entry.and_then(|entry| entry.eval);
    let raw_eval = tt_eval.unwrap_or_else(|| pos.get_eval());
    let stand_pat = raw_eval + pos.aggression(thread.stm, thread.eval);
    /*
    If not in check, we have a stand pat score which is the static eval of the current position.
    This is done as captures aren't necessarily the best moves.
    */
    if !in_check && stand_pat > alpha {
        alpha = stand_pat;
        highest_score = Some(stand_pat);
        if stand_pat >= beta {
            return stand_pat;
        }
    }

    let mut move_cnt = 0;
    let mut move_gen = QSearchMoveGen::new();
    while let Some(make_move) = move_gen.next(pos, &thread.history) {
        /*
        Prune all losing captures
        */
        if !compare_see(pos.board(), make_move, 0) {
            continue;
        }
        /*
        Fail high if SEE puts us above beta
        */
        if stand_pat + 1000 >= beta
            && compare_see(pos.board(), make_move, (beta - stand_pat + 192).raw())
        {
            return beta;
        }
        // Also prune neutral captures when static eval is low
        if stand_pat + 192 <= alpha && !compare_see(pos.board(), make_move, 1) {
            continue;
        }
        pos.make_move_fetch(make_move, |board| {
            shared_context.get_t_table().prefetch(board)
        });
        let search_score = q_search(
            pos,
            thread,
            shared_context,
            ply + 1,
            beta >> Next,
            alpha >> Next,
        );
        let score = search_score << Next;
        if highest_score.is_none() || score > highest_score.unwrap() {
            highest_score = Some(score);
        }
        if score > alpha {
            best_move = Some(make_move);
            alpha = score;
            if score >= beta {
                pos.unmake_move();
                break;
            }
        }
        pos.unmake_move();
        move_cnt += 1;
        if move_cnt >= 2 {
            break;
        }
    }

    if thread.abort {
        return Evaluation::min();
    }
    if let Some(highest_score) = highest_score {
        let entry_type = match () {
            _ if highest_score <= initial_alpha => Bounds::UpperBound,
            _ if highest_score >= beta => Bounds::LowerBound,
            _ => Bounds::Exact,
        };

        shared_context.get_t_table().set(
            pos.board(),
            0,
            entry_type,
            highest_score,
            best_move,
            raw_eval,
        );
    }
    highest_score.unwrap_or(alpha)
}
