use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use chess::{Board, ChessMove};

use crate::bm::bm_eval::eval::Evaluation;
use crate::bm::bm_runner::ab_consts::*;
use crate::bm::bm_runner::config::{GuiInfo, SearchMode, SearchStats};
use crate::bm::bm_search::move_entry::MoveEntry;
use crate::bm::bm_search::reduction::Reduction;
use crate::bm::bm_search::search;
use crate::bm::bm_search::search::Pv;
use crate::bm::bm_search::threshold::Threshold;
use crate::bm::bm_util::h_table::HistoryTable;
use crate::bm::bm_util::lookup::LookUp2d;
use crate::bm::bm_util::position::Position;
use crate::bm::bm_util::t_table::TranspositionTable;
use crate::bm::bm_util::window::Window;

use super::time::TimeManager;

pub const SEARCH_PARAMS: SearchParams = SearchParams {
    killer_move_cnt: KILLER_MOVE_CNT,
    threat_move_cnt: THREAT_MOVE_CNT,
    fail_cnt: FAIL_CNT,
    iid_depth: IID_DEPTH,
    rev_f_prune_depth: REV_F_PRUNE_DEPTH,
    fp: F_PRUNE_THRESHOLD,
    do_fp: DO_F_PRUNE,
    rev_fp: Threshold::new(REV_F_PRUNE_THRESHOLD_BASE, REV_F_PRUNE_THRESHOLD_FACTOR),
    do_rev_fp: DO_REV_F_PRUNE,
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
    fp: i16,
    do_fp: bool,
    rev_f_prune_depth: u32,
    rev_fp: Threshold,
    do_rev_fp: bool,
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
    delta_margin: i16,
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
    pub const fn get_delta(&self) -> i16 {
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
    pub const fn do_rev_f_prune(&self, depth: u32) -> bool {
        depth < self.rev_f_prune_depth
    }

    #[inline]
    pub const fn get_rev_fp(&self) -> &Threshold {
        &self.rev_fp
    }

    #[inline]
    pub const fn do_rev_fp(&self) -> bool {
        self.do_rev_fp
    }

    #[inline]
    pub const fn get_fp(&self) -> i16 {
        self.fp
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
    pub const fn do_lmp(&self) -> bool {
        self.do_lmp
    }
}

type LmrLookup = LookUp2d<u32, 32, 64>;
type LmpLookup = LookUp2d<usize, { LMP_DEPTH as usize }, 2>;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    start: Instant,
    time_manager: Arc<dyn TimeManager>,
    counter: u8,

    window: Window,
    t_table: Arc<TranspositionTable>,
    h_table: Arc<HistoryTable>,
    killer_moves: Vec<MoveEntry<{ SEARCH_PARAMS.get_k_move_cnt() }>>,
    threat_moves: Vec<MoveEntry<{ SEARCH_PARAMS.get_threat_move_cnt() }>>,
    lmr_lookup: Arc<LmrLookup>,
    lmp_lookup: Arc<LmpLookup>,
    tt_hits: u32,
    tt_misses: u32,
    eval: Evaluation,
    eval_stack: Vec<Evaluation>,
    sel_depth: u32,

    singular: bool,
}

impl SearchOptions {
    #[inline]
    pub fn abort_absolute(&mut self, depth: u32, nodes: u32) -> bool {
        self.time_manager.abort(self.start, depth, nodes)
    }

    #[inline]
    pub fn get_threat_table(&mut self) -> &mut Vec<MoveEntry<THREAT_MOVE_CNT>> {
        &mut self.threat_moves
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

    #[inline]
    pub fn get_last_eval(&self, ply: u32) -> Option<Evaluation> {
        if ply > 1 {
            Some(self.eval_stack[ply as usize - 2])
        } else {
            None
        }
    }

    #[inline]
    pub fn push_eval(&mut self, eval: Evaluation, ply: u32) {
        if ply as usize >= self.eval_stack.len() {
            self.eval_stack.push(eval);
        } else {
            self.eval_stack[ply as usize] = eval;
        }
    }

    #[inline]
    pub fn update_sel_depth(&mut self, ply: u32) {
        self.sel_depth = self.sel_depth.max(ply);
    }

    #[inline]
    pub fn singular(&self) -> bool {
        self.singular
    }

    #[inline]
    pub fn set_singular(&mut self, singular: bool) {
        self.singular = singular;
    }
}

pub struct AbRunner {
    search_options: SearchOptions,
    position: Position,
}

impl AbRunner {
    fn launch_searcher<SM: 'static + SearchMode + Send, Info: 'static + GuiInfo + Send>(
        &self,
        search_start: Instant,
        thread: u8,
    ) -> JoinHandle<(Option<ChessMove>, Evaluation, u32, u32)> {
        let mut nodes = 0;

        let mut search_options = self.search_options.clone();
        search_options.start = search_start;
        let mut position = self.position.clone();
        let mut debugger = SM::new(self.position.board());
        let gui_info = Info::new();
        std::thread::spawn(move || {
            let start_time = Instant::now();
            let mut best_move = None;
            let mut eval: Option<Evaluation> = None;
            let mut depth = 1_u32;
            'outer: loop {
                let mut fail_cnt = 0;
                search_options.window.reset();
                loop {
                    let (alpha, beta) = if eval.is_some()
                        && eval.unwrap().raw().abs() < 1000
                        && depth > 4
                        && fail_cnt < SEARCH_PARAMS.fail_cnt
                    {
                        search_options.window.get()
                    } else {
                        (Evaluation::min(), Evaluation::max())
                    };
                    let (make_move, score) = search::search::<Pv>(
                        &mut position,
                        &mut search_options,
                        0,
                        depth,
                        alpha,
                        beta,
                        &mut nodes,
                    );
                    if depth > 1 && search_options.abort_absolute(depth, nodes) {
                        break 'outer;
                    }
                    search_options.window.set(score);
                    if score > alpha && score < beta || score.is_mate() {
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
                        while let Some(analysis) = search_options.t_table.get(position.board()) {
                            pv.push(analysis.table_move());
                            position.make_move(analysis.table_move());
                            if pv.len() > depth as usize {
                                break;
                            }
                        }
                        for _ in 0..pv.len() {
                            position.unmake_move()
                        }
                        gui_info.print_info(
                            search_options.sel_depth,
                            depth,
                            eval,
                            start_time.elapsed(),
                            nodes,
                            &pv,
                        );
                    }
                }
                depth += 1;

                search_options.time_manager.deepen(
                    thread,
                    depth,
                    nodes,
                    search_options.eval,
                    best_move.unwrap(),
                    search_start.elapsed(),
                );
            }
            if let Some(evaluation) = eval {
                debugger.complete();
                (best_move, evaluation, depth, nodes)
            } else {
                panic!("# Search function has failed to evaluate the position");
            }
        })
    }

    pub fn new(board: Board, time_manager: Arc<dyn TimeManager>) -> Self {
        let mut position = Position::new(board);
        Self {
            search_options: SearchOptions {
                time_manager,
                window: Window::new(WINDOW_START, WINDOW_FACTOR, WINDOW_DIVISOR, WINDOW_ADD),
                t_table: Arc::new(TranspositionTable::new(2_usize.pow(21))),
                h_table: Arc::new(HistoryTable::new()),
                killer_moves: Vec::new(),
                threat_moves: Vec::new(),
                lmr_lookup: Arc::new(LookUp2d::new(|depth, mv| {
                    if depth == 0 || mv == 0 {
                        0
                    } else {
                        (LMR_BASE + (depth as f32).ln() * (mv as f32).ln() / LMR_DIV) as u32
                    }
                })),
                lmp_lookup: Arc::new(LookUp2d::new(|depth, improving| {
                    let mut x = LMP_OFFSET + depth as f32 * depth as f32 * LMP_FACTOR;
                    if improving == 0 {
                        x *= IMPROVING_DIVISOR;
                    }
                    x as usize
                })),
                tt_hits: 0,
                tt_misses: 0,
                eval: position.get_eval(),
                start: Instant::now(),
                counter: 0,
                eval_stack: vec![],
                sel_depth: 0,
                singular: false,
            },
            position,
        }
    }

    pub fn search<SM: 'static + SearchMode + Send, Info: 'static + GuiInfo + Send>(
        &self,
        threads: u8,
    ) -> (ChessMove, Evaluation, u32, u32) {
        let mut node_count = 0;
        let mut join_handlers = vec![];

        let search_start = Instant::now();
        //TODO: Research the effects of different depths
        for i in 0..threads {
            join_handlers.push(self.launch_searcher::<SM, Info>(search_start, i));
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

    pub fn raw_eval(&mut self) -> Evaluation {
        self.position.get_eval()
    }

    pub fn set_board_no_reset(&mut self, board: Board) {
        self.position = Position::new(board);
        self.search_options.eval = self.position.get_eval();
    }

    pub fn make_move_no_reset(&mut self, make_move: ChessMove) {
        self.position.make_move(make_move);
    }

    pub fn set_board(&mut self, board: Board) {
        self.search_options.h_table.for_all(|_| 0);
        self.search_options.t_table.clean();
        self.position = Position::new(board);
        self.search_options.eval = self.position.get_eval();
    }

    pub fn make_move(&mut self, make_move: ChessMove) {
        self.position.make_move(make_move);
    }

    pub fn get_board(&self) -> &Board {
        self.position.board()
    }
}
