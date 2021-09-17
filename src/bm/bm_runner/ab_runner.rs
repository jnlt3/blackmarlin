use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use chess::{Board, ChessMove};

use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_runner::ab_consts::*;
use crate::bm::bm_runner::config::{GuiInfo, SearchMode, SearchStats};
use crate::bm::bm_runner::runner::Runner;
use crate::bm::bm_search::move_entry::MoveEntry;
use crate::bm::bm_search::reduction::Reduction;
use crate::bm::bm_search::search;
use crate::bm::bm_search::search::Pv;
use crate::bm::bm_search::threshold::Threshold;
use crate::bm::bm_util::c_hist::CMoveHistoryTable;
use crate::bm::bm_util::c_move::CounterMoveTable;
use crate::bm::bm_util::ch_table::CaptureHistoryTable;
use crate::bm::bm_util::evaluator::Evaluator;
use crate::bm::bm_util::h_table::HistoryTable;
use crate::bm::bm_util::lookup::{LookUp, LookUp2d};
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::TranspositionTable;
use crate::bm::bm_util::window::Window;

use super::time::TimeManager;

pub const SEARCH_PARAMS: SearchParams = SearchParams {
    killer_move_cnt: KILLER_MOVE_CNT,
    threat_move_cnt: THREAT_MOVE_CNT,
    fail_cnt: FAIL_CNT,
    iid_depth: IID_DEPTH,
    f_prune_depth: F_PRUNE_DEPTH,
    fp: Threshold::new(F_PRUNE_THRESHOLD_BASE, F_PRUNE_THRESHOLD_FACTOR),
    do_fp: DO_F_PRUNE,
    nmp: Reduction::new(
        NULL_MOVE_REDUCTION_BASE,
        NULL_MOVE_REDUCTION_FACTOR,
        NULL_MOVE_REDUCTION_DIVISOR,
    ),
    do_nmp: DO_NULL_MOVE_REDUCTION,
    iid: Reduction::new(IID_BASE, IID_FACTOR, IID_DIVISOR),
    do_iid: DO_IID,
    lmr_pv: LMR_PV,
    lmr_depth: LMR_DEPTH,
    do_lmr: DO_LMR,
    lmp_depth: LMP_DEPTH,
    do_lmp: DO_LMP,
    q_search_depth: QUIESCENCE_SEARCH_DEPTH,
    delta_margin: DELTA_MARGIN,
    do_dp: DO_DELTA_PRUNE,
    do_see_prune: DO_SEE_PRUNE,
};

#[derive(Debug, Clone)]
pub struct SearchParams {
    killer_move_cnt: usize,
    threat_move_cnt: usize,
    fail_cnt: u8,
    iid_depth: u32,
    f_prune_depth: u32,
    fp: Threshold,
    do_fp: bool,
    nmp: Reduction,
    do_nmp: bool,
    iid: Reduction,
    do_iid: bool,
    lmr_pv: u32,
    lmr_depth: u32,
    do_lmr: bool,
    lmp_depth: u32,
    do_lmp: bool,
    q_search_depth: u32,
    delta_margin: i32,
    do_dp: bool,
    do_see_prune: bool,
}

impl SearchParams {
    #[inline]
    pub const fn get_k_move_cnt(&self) -> usize {
        self.killer_move_cnt
    }

    #[inline]
    pub const fn get_threat_move_cnt(&self) -> usize {
        self.threat_move_cnt
    }

    #[inline]
    pub const fn get_q_search_depth(&self) -> u32 {
        self.q_search_depth
    }

    #[inline]
    pub const fn get_delta(&self) -> i32 {
        self.delta_margin
    }

    #[inline]
    pub const fn do_dp(&self) -> bool {
        self.do_dp
    }

    #[inline]
    pub const fn do_see_prune(&self) -> bool {
        self.do_see_prune
    }

    #[inline]
    pub const fn do_f_prune(&self, depth: u32) -> bool {
        depth < self.f_prune_depth
    }

    #[inline]
    pub const fn get_fp(&self) -> &Threshold {
        &self.fp
    }

    #[inline]
    pub const fn do_fp(&self) -> bool {
        self.do_fp
    }

    #[inline]
    pub const fn get_nmp(&self) -> &Reduction {
        &self.nmp
    }

    #[inline]
    pub const fn do_nmp(&self) -> bool {
        self.do_nmp
    }

    #[inline]
    pub const fn get_iid(&self) -> &Reduction {
        &self.iid
    }

    #[inline]
    pub const fn do_iid(&self, depth: u32) -> bool {
        self.do_iid && depth > self.iid_depth
    }

    #[inline]
    pub const fn get_lmr_pv(&self) -> u32 {
        self.lmr_pv
    }

    #[inline]
    pub const fn do_lmr(&self, depth: u32) -> bool {
        self.do_lmr && depth > self.lmr_depth
    }

    #[inline]
    pub const fn do_lmp(&self, depth: u32) -> bool {
        self.do_lmp && depth < self.lmp_depth
    }
}

type LmrLookup = LookUp2d<u32, 32, 64>;
type LmpLookup = LookUp<usize, { LMP_DEPTH as usize }>;

#[derive(Debug, Clone)]
pub struct SearchOptions<Eval: 'static + Evaluator + Clone + Send> {
    evaluator: Eval,
    search_start: Instant,
    time_manager: Arc<dyn TimeManager>,
    window: Window,
    t_table: Arc<TranspositionTable>,
    h_table: Arc<HistoryTable>,
    ch_table: Arc<CaptureHistoryTable>,
    c_hist: Arc<CMoveHistoryTable>,
    c_table: Arc<CounterMoveTable>,
    killer_moves: Vec<MoveEntry<KILLER_MOVE_CNT>>,
    threat_moves: Vec<MoveEntry<THREAT_MOVE_CNT>>,
    lmr_lookup: Arc<LmrLookup>,
    lmp_lookup: Arc<LmpLookup>,
    tt_hits: u32,
    tt_misses: u32,
    eval: Evaluation,
}

impl<Eval: 'static + Evaluator + Clone + Send> SearchOptions<Eval> {
    #[inline]
    pub fn abort(&self) -> bool {
        self.time_manager.abort(self.search_start.elapsed())
    }

    #[inline]
    pub fn get_threat_table(&mut self) -> &mut Vec<MoveEntry<THREAT_MOVE_CNT>> {
        &mut self.threat_moves
    }

    #[inline]
    pub fn eval(&mut self) -> &mut Eval {
        &mut self.evaluator
    }

    #[inline]
    pub fn get_t_table(&self) -> &Arc<TranspositionTable> {
        &self.t_table
    }

    #[inline]
    pub fn get_h_table(&self) -> &Arc<HistoryTable> {
        &self.h_table
    }

    #[inline]
    pub fn get_ch_table(&self) -> &Arc<CaptureHistoryTable> {
        &self.ch_table
    }

    #[inline]
    pub fn get_c_hist(&self) -> &Arc<CMoveHistoryTable> {
        &self.c_hist
    }

    #[inline]
    pub fn get_c_table(&self) -> &Arc<CounterMoveTable> {
        &self.c_table
    }

    #[inline]
    pub fn get_k_table(&mut self) -> &mut Vec<MoveEntry<KILLER_MOVE_CNT>> {
        &mut self.killer_moves
    }

    #[inline]
    pub fn get_lmr_lookup(&self) -> &Arc<LmrLookup> {
        &self.lmr_lookup
    }

    #[inline]
    pub fn get_lmp_lookup(&self) -> &Arc<LmpLookup> {
        &self.lmp_lookup
    }

    #[inline]
    pub fn tt_hits(&mut self) -> &mut u32 {
        &mut self.tt_hits
    }

    #[inline]
    pub fn tt_misses(&mut self) -> &mut u32 {
        &mut self.tt_misses
    }
}

pub struct AbRunner<Eval: 'static + Evaluator + Clone + Send> {
    search_options: SearchOptions<Eval>,
    position: Position,
}

impl<Eval: 'static + Evaluator + Clone + Send> AbRunner<Eval> {
    fn launch_searcher<SM: 'static + SearchMode + Send, Info: 'static + GuiInfo + Send>(
        &mut self,
        thread: u8,
        incr: u32,
    ) -> JoinHandle<(Option<ChessMove>, Evaluation, u32, u32)> {
        let mut nodes = 0;

        let mut search_options = self.search_options.clone();
        let mut position = self.position.clone();
        let mut debugger = SM::new(self.position.board());
        let gui_info = Info::new();
        std::thread::spawn(move || {
            let start_time = Instant::now();
            let mut best_move = None;
            let mut eval = None;
            let mut depth = 1;
            'outer: loop {
                search_options.window.set(search_options.eval);
                let mut fail_cnt = 0;
                let (alpha, beta) = if fail_cnt < SEARCH_PARAMS.fail_cnt {
                    search_options.window.get()
                } else {
                    (Evaluation::min(), Evaluation::max())
                };
                loop {
                    let (make_move, score) = search::search::<Pv, Eval>(
                        &mut position,
                        &mut search_options,
                        0,
                        depth,
                        alpha,
                        beta,
                        &mut nodes,
                    );
                    if depth > 1 && search_options.abort() {
                        break 'outer;
                    }
                    if fail_cnt >= SEARCH_PARAMS.fail_cnt
                        || (score > alpha && score < beta)
                        || score.is_mate()
                    {
                        search_options.eval = score;
                        best_move = make_move;
                        eval = Some(score);
                        break;
                    } else {
                        fail_cnt += 1;
                        if score <= alpha {
                            search_options.window.fail_low();
                        } else {
                            search_options.window.fail_high();
                        }
                    }
                }
                debugger.push(SearchStats::new(
                    start_time.elapsed().as_millis(),
                    depth,
                    eval,
                    best_move,
                ));
                if let Some(eval) = eval {
                    if let Some(best_move) = best_move {
                        let best_move = best_move;
                        let mut pv = vec![best_move];
                        position.make_move(best_move);
                        while let Some(analysis) = search_options.t_table.get(&position) {
                            pv.push(analysis.table_move());
                            position.make_move(analysis.table_move());
                            if pv.len() > depth as usize {
                                break;
                            }
                        }
                        for _ in 0..pv.len() {
                            position.unmake_move()
                        }
                        gui_info.print_info(depth, eval, start_time.elapsed(), nodes, &pv);
                    }
                }
                if search_options.eval.is_mate() {
                    break;
                }
                depth += incr;
                //As far as time manager is concerned Option<ChessMove> is no different than ChessMove however None == None doesn't apply
                search_options.time_manager.deepen(
                    thread,
                    depth,
                    nodes,
                    search_options.eval,
                    best_move.unwrap(),
                    search_options.search_start.elapsed(),
                )
            }
            if let Some(evaluation) = eval {
                debugger.complete();
                (best_move, evaluation, depth, nodes)
            } else {
                panic!("# Search function has failed to evaluate the position");
            }
        })
    }
}

impl<Eval: 'static + Evaluator + Clone + Send> Runner<Eval> for AbRunner<Eval> {
    fn new(board: Board, time_manager: Arc<dyn TimeManager>) -> Self {
        let position = Position::new(board);
        let mut evaluator = Eval::new();
        Self {
            search_options: SearchOptions {
                search_start: Instant::now(),
                time_manager,
                window: Window::new(WINDOW_START, WINDOW_FACTOR, WINDOW_DIVISOR, WINDOW_ADD),
                t_table: Arc::new(TranspositionTable::new(2_usize.pow(21))),
                h_table: Arc::new(HistoryTable::new()),
                ch_table: Arc::new(CaptureHistoryTable::new()),
                c_hist: Arc::new(CMoveHistoryTable::new()),
                c_table: Arc::new(CounterMoveTable::new()),
                killer_moves: Vec::new(),
                threat_moves: Vec::new(),
                lmr_lookup: Arc::new(LookUp2d::new(|depth, mv| {
                    if depth == 0 || mv == 0 {
                        0
                    } else {
                        (LMR_BASE + (depth as f32).ln() * (mv as f32).ln() / LMR_DIV) as u32
                    }
                })),
                lmp_lookup: Arc::new(LookUp::new(|depth| {
                    (LMP_OFFSET + depth as f32 * depth as f32 * LMP_FACTOR) as usize
                })),
                tt_hits: 0,
                tt_misses: 0,
                eval: evaluator.evaluate(&position),
                evaluator,
            },
            position,
        }
    }

    fn search<SM: 'static + SearchMode + Send, Info: 'static + GuiInfo + Send>(
        &mut self,
        threads: u8,
    ) -> (ChessMove, Evaluation, u32, u32) {
        let mut node_count = 0;
        self.search_options.search_start = Instant::now();
        let mut join_handlers = vec![];

        //TODO: Research the effects of different depths
        for i in 0..threads {
            join_handlers.push(self.launch_searcher::<SM, Info>(i, (i % 2 + 1) as u32));
        }
        let mut final_move = None;
        let mut final_eval = None;
        let mut max_depth = 0;
        for join_handler in join_handlers {
            let (best_move, eval, depth, nodes) = join_handler.join().unwrap();
            node_count += nodes;
            if let Some(best_move) = best_move {
                if final_eval.is_none() || eval > final_eval.unwrap() {
                    final_move = Some(best_move);
                    final_eval = Some(eval);
                    max_depth = u32::max(max_depth, depth);
                }
            } else {
                println!("# Move generation failed");
            }
        }
        if final_move.is_none() {
            panic!("# All move generation has failed");
        } else if final_eval.is_none() {
            panic!("# All evaluations have failed");
        }
        (
            final_move.unwrap(),
            final_eval.unwrap(),
            max_depth,
            node_count,
        )
    }

    fn raw_eval(&mut self) -> Evaluation {
        self.search_options.evaluator.evaluate(&self.position)
    }

    fn set_board(&mut self, board: Board) {
        self.search_options.h_table.for_all(|_| 0);
        self.search_options.c_hist.for_all(|_| 0);
        self.search_options.ch_table.for_all(|_| 0);
        self.search_options.c_table.clear();
        self.search_options.t_table.clean();
        self.position = Position::new(board);
        self.search_options.eval().clear_cache();
        self.search_options.eval = self.search_options.evaluator.evaluate(&self.position);
    }

    fn make_move(&mut self, make_move: ChessMove) {
        self.search_options.h_table.for_all(|weight| weight / 8);
        self.search_options.c_hist.for_all(|weight| weight / 8);
        self.search_options.ch_table.for_all(|weight| weight / 8);
        self.position.make_move(make_move);
        self.search_options.eval().clear_cache();
    }

    fn pv(&mut self, pv_len: usize) -> Vec<ChessMove> {
        let mut moves = vec![];
        for _ in 0..pv_len {
            if let Some(make_move) = self.search_options.get_t_table().get(&self.position) {
                moves.push(make_move.table_move());
                self.position.make_move(make_move.table_move());
            }
        }
        for _ in 0..moves.len() {
            self.position.unmake_move();
        }
        moves
    }

    fn get_board(&self) -> &Board {
        &self.position.board()
    }
}
