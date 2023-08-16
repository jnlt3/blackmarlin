use arrayvec::ArrayVec;
use cozy_chess::{Board, Color, Move, Piece};

use crate::bm::bm_runner::ab_runner::{LocalContext, MoveData, SharedContext, MAX_PLY};
use crate::bm::bm_util::eval::Depth::Next;
use crate::bm::bm_util::eval::Evaluation;
use crate::bm::bm_util::history::HistoryIndices;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::EntryType;
use crate::bm::bm_util::t_table::EntryType::{Exact, LowerBound, UpperBound};

use super::move_gen::{OrderedMoveGen, Phase, QSearchMoveGen};
use super::see::{self, compare_see};

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
    depth <= 8
}

const fn rev_fp(depth: u32, improving: bool) -> i16 {
    depth as i16 * 54 - improving as i16 * 49
}

const fn do_razor(depth: u32) -> bool {
    depth <= 3
}

const fn razor(depth: u32) -> i16 {
    depth as i16 * 200
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
        && !(nstm_threat && depth <= 8)
        && eval >= beta
        && (board.pieces(Piece::Pawn) | board.pieces(Piece::King)) != board.occupied()
}

fn nmp_depth(depth: u32, eval: i16, beta: i16) -> u32 {
    assert!(eval >= beta);
    let r = 4 + depth / 3 + ((eval - beta) / 206) as u32;
    depth.saturating_sub(r).max(1)
}

const fn iir(depth: u32) -> u32 {
    if depth >= 2 {
        1
    } else {
        0
    }
}

const fn fp(depth: u32) -> i16 {
    depth as i16 * 62
}

const fn see_fp(depth: u32) -> i16 {
    depth as i16 * 81
}

const fn hp(depth: u32) -> i32 {
    -((depth * depth) as i32) * 71 / 10
}

const fn history_lmr(history: i16) -> i16 {
    history / 92
}

pub fn search<Search: SearchType>(
    pos: &mut Position,
    local_context: &mut LocalContext,
    shared_context: &SharedContext,
    ply: u32,
    mut depth: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> Evaluation {
    local_context.search_stack_mut()[ply as usize].pv_len = 0;

    if ply != 0 && (local_context.abort() || shared_context.abort_search(local_context.nodes())) {
        local_context.trigger_abort();
        return Evaluation::min();
    }

    local_context.update_sel_depth(ply);
    if ply != 0 && pos.forced_draw(ply) {
        local_context.increment_nodes();
        return Evaluation::new(0);
    }

    /*
    At depth 0, we run Quiescence Search
    */
    if depth == 0 || ply >= MAX_PLY {
        return q_search(pos, local_context, shared_context, ply, alpha, beta);
    }

    let skip_move = local_context.search_stack()[ply as usize].skip_move;
    let tt_entry = match skip_move {
        Some(_) => None,
        None => shared_context.get_t_table().get(pos.board()),
    };

    local_context.increment_nodes();

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
        *local_context.tt_hits() += 1;
        best_move = Some(entry.table_move());
        if !Search::PV && entry.depth() >= depth {
            let score = entry.score();
            match entry.entry_type() {
                Exact => {
                    return score;
                }
                LowerBound => {
                    if score >= beta {
                        return score;
                    }
                }
                UpperBound => {
                    if score <= alpha {
                        return score;
                    }
                }
            }
        }
    } else {
        *local_context.tt_misses() += 1;
    }

    let in_check = !pos.board().checkers().is_empty();

    let eval = match skip_move {
        Some(_) => local_context.search_stack()[ply as usize].eval,
        None => pos.get_eval(local_context.stm(), local_context.eval()),
    };

    local_context.search_stack_mut()[ply as usize].eval = eval;

    let prev_move_eval = match ply {
        2.. => Some(local_context.search_stack()[ply as usize - 2].eval),
        _ => None,
    };
    let improving = match prev_move_eval {
        Some(prev_move_eval) => !in_check && eval > prev_move_eval,
        None => false,
    };

    let (w_threats, b_threats) = pos.threats();
    let nstm_threats = match pos.board().side_to_move() {
        Color::White => b_threats,
        Color::Black => w_threats,
    };
    if !Search::PV && !in_check && skip_move.is_none() {
        /*
        Reverse Futility Pruning:
        If in a non PV node and evaluation is higher than beta + a depth dependent margin
        we assume we can at least achieve beta
        */
        if do_rev_fp(depth) && eval - rev_fp(depth, improving && nstm_threats.is_empty()) >= beta {
            return eval;
        }

        let razor_margin = razor(depth);
        if do_razor(depth) && eval + razor_margin <= alpha {
            let zw = alpha - razor_margin;
            let q_search = q_search(pos, local_context, shared_context, ply, zw, zw + 1);
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
        if do_nmp::<Search>(
            pos.board(),
            depth,
            eval.raw(),
            beta.raw(),
            !nstm_threats.is_empty(),
        ) && pos.null_move()
        {
            local_context.search_stack_mut()[ply as usize].move_played = None;

            let nmp_depth = nmp_depth(depth, eval.raw(), beta.raw());
            let zw = beta >> Next;
            let search_score = search::<NoNm>(
                pos,
                local_context,
                shared_context,
                ply + 1,
                nmp_depth,
                zw,
                zw + 1,
            );
            pos.unmake_move();
            let score = search_score << Next;
            if score >= beta {
                let mut verified = depth < 10;
                if !verified {
                    let verification = search::<NoNm>(
                        pos,
                        local_context,
                        shared_context,
                        ply + 1,
                        nmp_depth,
                        alpha,
                        beta,
                    );
                    verified = verification >= beta;
                }
                if verified {
                    return score;
                }
            }
        }
    }

    if tt_entry.is_none() {
        depth -= iir(depth)
    }

    if let Some(entry) = local_context.get_k_table().get_mut(ply as usize + 1) {
        entry.clear();
    }

    let mut highest_score = None;

    let prev_move = match ply > 1 {
        true => local_context.search_stack()[ply as usize - 2].move_played,
        false => None,
    };
    let opp_move = match ply != 0 {
        true => local_context.search_stack()[ply as usize - 1].move_played,
        false => None,
    };

    let killers = local_context.get_k_table()[ply as usize];
    let mut move_gen = OrderedMoveGen::new(pos.board(), best_move, killers);

    let mut moves_seen = 0;
    let mut move_exists = false;

    let mut quiets = ArrayVec::<Move, 64>::new();
    let mut captures = ArrayVec::<Move, 64>::new();

    let hist_indices = HistoryIndices::new(opp_move, prev_move);
    while let Some(make_move) = move_gen.next(pos, local_context.get_hist(), &hist_indices) {
        if Some(make_move) == skip_move {
            continue;
        }

        move_exists = true;
        let is_capture = pos
            .board()
            .colors(!pos.board().side_to_move())
            .has(make_move.to);

        let h_score = match is_capture {
            true => local_context.get_hist().get_capture(pos, make_move),
            false => {
                (local_context.get_hist().get_quiet(pos, make_move)
                    + local_context
                        .get_hist()
                        .get_counter_move(pos, &hist_indices, make_move)
                        .unwrap_or_default())
                    / 2
            }
        };
        local_context.search_stack_mut()[ply as usize + 1].pv_len = 0;

        let mut extension = 0;
        let mut score;

        /*
        Singular Extensions:
        If a move can't be beaten by any other move, we assume the move
        is singular (only solution) and extend in order to get a more accurate
        estimation of best move/eval
        */
        if let Some(entry) = tt_entry {
            if moves_seen == 0
                && entry.table_move() == make_move
                && ply != 0
                && !entry.score().is_mate()
                && entry.depth() + 2 >= depth
                && matches!(entry.entry_type(), EntryType::LowerBound | EntryType::Exact)
            {
                let s_beta = entry.score() - depth as i16;
                local_context.search_stack_mut()[ply as usize].skip_move = Some(make_move);

                let multi_cut = depth >= 5;
                let s_score = match multi_cut {
                    true => search::<Search::Zw>(
                        pos,
                        local_context,
                        shared_context,
                        ply,
                        depth / 2 - 1,
                        s_beta - 1,
                        s_beta,
                    ),
                    false => eval,
                };

                local_context.search_stack_mut()[ply as usize].skip_move = None;
                if s_score < s_beta {
                    extension = 1;
                    if !Search::PV && multi_cut && s_score + 19 < s_beta {
                        extension += 1;
                    }
                    local_context.get_hist_mut().update_history(
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
                }
            }
        }

        let non_mate_line = highest_score.map_or(false, |s: Evaluation| !s.is_mate());
        /*
        In non-PV nodes If a move isn't good enough to beat alpha - a static margin
        we assume it's safe to prune this move
        */
        let do_fp = !Search::PV && non_mate_line && moves_seen > 0 && !is_capture && depth <= 5;

        if do_fp && eval + fp(depth) <= alpha {
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

        /*
        In low depth, non-PV nodes, we assume it's safe to prune a move
        if it has very low history
        */
        let do_hp = !Search::PV && non_mate_line && moves_seen > 0 && depth <= 7 && eval <= alpha;

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
            && depth <= 7
            && move_gen.phase() > Phase::GoodCaptures;

        let see_margin = (alpha - eval - see_fp(depth) + 1).raw();
        if do_see_prune && (see_margin > 0 || !compare_see(pos.board(), make_move, see_margin)) {
            continue;
        }

        local_context.search_stack_mut()[ply as usize].move_played =
            Some(MoveData::from_move(pos.board(), make_move));
        pos.make_move(make_move);
        shared_context.get_t_table().prefetch(pos.board());
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
        let mut reduction = shared_context
            .get_lmr_lookup()
            .get(depth as usize, moves_seen) as i16;

        if moves_seen > 0 {
            /*
            If a move is quiet, we already have information on this move
            in the history table. If history score is high, we reduce
            less and if history score is low we reduce more.
            */

            reduction -= history_lmr(h_score);
            if ply <= (depth + ply) / 3 {
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
            reduction = reduction.min(depth as i16 - 2).max(0);
        }

        let lmr_depth = (depth as i16 - reduction) as u32;

        if moves_seen == 0 {
            let search_score = search::<Search>(
                pos,
                local_context,
                shared_context,
                ply + 1,
                depth - 1 + extension,
                beta >> Next,
                alpha >> Next,
            );
            score = search_score << Next;
        } else {
            //Reduced Search/Zero Window if no reduction
            let zw = alpha >> Next;

            let lmr_score = search::<Search::Zw>(
                pos,
                local_context,
                shared_context,
                ply + 1,
                lmr_depth - 1 + extension,
                zw - 1,
                zw,
            );
            score = lmr_score << Next;

            /*
            If no reductions occured in LMR we don't waste time re-searching
            otherwise, we run a full depth search to attempt a fail low
            */
            if lmr_depth < depth && score > alpha {
                let zw_score = search::<Search::Zw>(
                    pos,
                    local_context,
                    shared_context,
                    ply + 1,
                    depth - 1 + extension,
                    zw - 1,
                    zw,
                );
                score = zw_score << Next;
            }
            /*
            If we don't get a fail low, this means the move has to be searched fully
            */
            if Search::PV && score > alpha {
                let search_score = search::<Search>(
                    pos,
                    local_context,
                    shared_context,
                    ply + 1,
                    depth - 1 + extension,
                    beta >> Next,
                    alpha >> Next,
                );
                score = search_score << Next;
            }
        }

        pos.unmake_move();
        moves_seen += 1;

        if highest_score.is_none() || score > highest_score.unwrap() {
            highest_score = Some(score);
            best_move = Some(make_move);
            if score > alpha {
                if (Search::PV || (ply == 0 && moves_seen == 1)) && !local_context.abort() {
                    let (child_pv, len) = {
                        let child = &local_context.search_stack()[ply as usize + 1];
                        (child.pv, child.pv_len)
                    };
                    local_context.search_stack_mut()[ply as usize]
                        .update_pv(make_move, &child_pv[..len]);
                }
                if score >= beta {
                    if !local_context.abort() {
                        let amt = depth + (eval <= alpha) as u32 + (score - 50 > beta) as u32;
                        if !is_capture {
                            let killer_table = local_context.get_k_table();
                            killer_table[ply as usize].push(make_move);
                        }
                        local_context.get_hist_mut().update_history(
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

    if skip_move.is_none() && !local_context.abort() {
        if let Some(final_move) = &best_move {
            let entry_type = match () {
                _ if highest_score <= initial_alpha => UpperBound,
                _ if highest_score >= beta => LowerBound,
                _ => Exact,
            };
            shared_context.get_t_table().set(
                pos.board(),
                depth,
                entry_type,
                highest_score,
                *final_move,
            );
        }
    }
    highest_score
}

/*
Quiescence Search is a form of search that only searches tactical moves to achieve a quiet position.
This is done as the static evaluation function isn't suited to detecting tactical aspects of the position.
*/

pub fn q_search(
    pos: &mut Position,
    local_context: &mut LocalContext,
    shared_context: &SharedContext,
    ply: u32,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> Evaluation {
    if local_context.abort() || shared_context.abort_search(local_context.nodes()) {
        local_context.trigger_abort();
        return Evaluation::min();
    }

    local_context.increment_nodes();

    local_context.update_sel_depth(ply);
    if ply >= MAX_PLY {
        return pos.get_eval(local_context.stm(), local_context.eval());
    }

    let initial_alpha = alpha;
    let tt_entry = shared_context.get_t_table().get(pos.board());
    if let Some(entry) = tt_entry {
        match entry.entry_type() {
            LowerBound => {
                if entry.score() >= beta {
                    return entry.score();
                }
            }
            Exact => return entry.score(),
            UpperBound => {
                if entry.score() <= alpha {
                    return entry.score();
                }
            }
        }
    }

    let mut highest_score = None;
    let mut best_move = None;
    let in_check = !pos.board().checkers().is_empty();

    let stand_pat = pos.get_eval(local_context.stm(), local_context.eval());
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

    let mut move_gen = QSearchMoveGen::new();
    while let Some((make_move, see)) = move_gen.next(pos, local_context.get_hist()) {
        let is_capture = pos
            .board()
            .colors(!pos.board().side_to_move())
            .has(make_move.to);
        if in_check || is_capture {
            /*
            SEE beta cutoff: (Koivisto)
            If SEE considerably improves evaluation above beta, we can return beta early
            */
            if stand_pat + see - 193 >= beta {
                return beta;
            }
            if stand_pat + 200 <= alpha && see <= 0 {
                continue;
            }
            pos.make_move(make_move);
            let search_score = q_search(
                pos,
                local_context,
                shared_context,
                ply + 1,
                beta >> Next,
                alpha >> Next,
            );
            let score = search_score << Next;
            if highest_score.is_none() || score > highest_score.unwrap() {
                highest_score = Some(score);
                best_move = Some(make_move);
            }
            if score > alpha {
                alpha = score;
                if score >= beta {
                    pos.unmake_move();
                    break;
                }
            }
            pos.unmake_move();
        }
    }
    if let Some((best_move, highest_score)) = best_move.zip(highest_score) {
        let entry_type = match () {
            _ if highest_score <= initial_alpha => UpperBound,
            _ if highest_score >= beta => LowerBound,
            _ => Exact,
        };

        shared_context
            .get_t_table()
            .set(pos.board(), 0, entry_type, highest_score, best_move);
    }
    highest_score.unwrap_or(alpha)
}
