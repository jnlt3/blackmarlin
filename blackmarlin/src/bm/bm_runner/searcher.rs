use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use cozy_chess::{Board, Color, Move, Piece, Square};

use crate::bm::{
    bm_runner::time::TimeManager,
    bm_search::{
        move_entry::MoveEntry,
        search::{self, Pv},
    },
    bm_util::{
        eval::Evaluation, history::History, lookup::LookUp2d, position::Position,
        t_table::TranspositionTable, window::Window,
    },
};

pub const MAX_PLY: u32 = 128;
pub type LmrLookup = LookUp2d<u32, 32, 64>;
pub type LmpLookup = LookUp2d<usize, 16, 2>;

#[derive(Debug, Copy, Clone)]
pub struct MoveData {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
    pub piece: Piece,
    pub capture: bool,
}

impl MoveData {
    pub fn from_move(board: &Board, make_move: Move) -> Self {
        Self {
            from: make_move.from,
            to: make_move.to,
            promotion: make_move.promotion,
            piece: board.piece_on(make_move.from).unwrap(),
            capture: board.colors(!board.side_to_move()).has(make_move.to),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackEntry {
    pub raw_eval: Evaluation,
    pub aggr: i16,
    pub corr: i16,
    pub skip_move: Option<Move>,
    pub move_played: Option<MoveData>,
    pub pv: [Option<Move>; MAX_PLY as usize + 1],
    pub pv_len: usize,
}

impl StackEntry {
    pub fn full_eval(&self) -> Evaluation {
        self.raw_eval + self.aggr + self.corr
    }

    pub fn update_pv(&mut self, best_move: Move, child_pv: &[Option<Move>]) {
        self.pv[0] = Some(best_move);
        for (pv, &child) in self.pv[1..].iter_mut().zip(child_pv) {
            *pv = child;
        }
        self.pv_len = child_pv.len() + 1;
    }
}

pub struct Searcher {
    pub context: Arc<SharedCtx>,

    workers: Vec<(SyncSender<Message>, JoinHandle<()>)>,
}

pub struct SharedCtx {
    pub time_manager: TimeManager,
    pub t_table: TranspositionTable,
    pub lmr_lookup: LmrLookup,
    pub lmp_lookup: LmpLookup,
}

pub struct ThreadData {
    pub thread_id: usize,
    /// Search Stack
    pub ss: Vec<StackEntry>,
    pub history: History,
    pub killer_moves: Vec<MoveEntry>,
}

pub struct ThreadCtx<'a> {
    pub thread_id: usize,
    pub tm: &'a TimeManager,
    pub ss: &'a mut [StackEntry],
    pub history: &'a History,
    pub killers: &'a mut [MoveEntry],
    pub nodes: Arc<AtomicU64>,
    pub root_nodes: [[u64; Square::NUM]; Square::NUM],

    pub window: Window,
    pub eval: Evaluation,
    pub stm: Color,

    pub abort: bool,

    pub tt_hits: u32,
    pub tt_misses: u32,
    pub sel_depth: u32,

    start: Instant,
}

impl SharedCtx {
    pub fn new_with_hash(hash_size: usize) -> Self {
        SharedCtx {
            time_manager: TimeManager::new(),
            t_table: TranspositionTable::new(hash_size * 1024 * 1024 / 12),
            lmr_lookup: LookUp2d::new(|depth, mv| {
                if depth == 0 || mv == 0 {
                    0
                } else {
                    (0.50 + (depth as f32).ln() * (mv as f32).ln() / 2.05) as u32
                }
            }),
            lmp_lookup: LookUp2d::new(|depth, improving| {
                let mut x = 2.97 + depth as f32 * depth as f32;
                if improving == 0 {
                    x /= 1.94;
                }
                x as usize
            }),
        }
    }
}

impl<'a> ThreadCtx<'a> {
    pub fn nodes(&self) -> u64 {
        self.nodes.load(Ordering::Relaxed)
    }

    pub fn trigger_abort(&mut self) {
        self.abort = true;
    }

    pub fn update_sel_depth(&mut self, ply: u32) {
        self.sel_depth = u32::max(self.sel_depth, ply);
    }

    pub fn increment_nodes(&self) {
        self.nodes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn abort_search(&self, nodes: u64) -> bool {
        if !nodes.is_multiple_of(1024) {
            return false;
        }
        self.tm.abort_search(self.start)
    }
}

impl ThreadData {
    pub fn new_with_id(thread_id: usize) -> Self {
        Self {
            thread_id,
            ss: vec![
                StackEntry {
                    raw_eval: Evaluation::new(0),
                    skip_move: None,
                    move_played: None,
                    pv: [None; MAX_PLY as usize + 1],
                    pv_len: 0,
                    aggr: 0,
                    corr: 0,
                };
                MAX_PLY as usize + 1
            ],
            history: History::new_zeroed(),
            killer_moves: vec![MoveEntry::new(); MAX_PLY as usize + 1],
        }
    }

    pub fn clear(&mut self) {
        self.history = History::new_zeroed();
        self.killer_moves
            .iter_mut()
            .for_each(|value| *value = MoveEntry::new());
        self.ss = vec![
            StackEntry {
                raw_eval: Evaluation::new(0),
                skip_move: None,
                move_played: None,
                pv: [None; MAX_PLY as usize + 1],
                pv_len: 0,
                aggr: 0,
                corr: 0,
            };
            MAX_PLY as usize + 1
        ];
    }
}

#[derive(Debug, Clone)]
pub struct SearchCtx {
    pub root: Position,
    pub start: Instant,
}

enum Message {
    Search(Arc<SharedCtx>, Box<SearchCtx>, Arc<dyn SearchObserver>),
    Quit,
    Wait,
    Clear,
}

impl Searcher {
    pub fn new(hash_size: usize, threads: usize) -> Self {
        let ctx = SharedCtx::new_with_hash(hash_size);
        Self {
            context: Arc::new(ctx),
            workers: (0..threads).map(Self::spawn_worker).collect(),
        }
    }

    pub fn set_hash(&mut self, hash_size: usize) {
        let context = Arc::get_mut(&mut self.context).unwrap();
        context.t_table = TranspositionTable::new(hash_size * 1024 * 1024 / 12);
    }

    pub fn set_threads(&mut self, threads: usize) {
        self.stop();
        if threads >= self.workers.len() {
            self.workers
                .extend((self.workers.len()..threads).map(Self::spawn_worker));
        } else {
            for (sender, handle) in self.workers.drain(threads..) {
                sender.send(Message::Quit).unwrap();
                handle.join().unwrap();
            }
        }
    }

    pub fn threads(&self) -> usize {
        self.workers.len()
    }

    pub fn search(&self, root: Position, observer: Arc<dyn SearchObserver>) {
        let search_ctx = SearchCtx {
            root,
            start: Instant::now(),
        };
        for (worker, _) in &self.workers {
            worker
                .send(Message::Search(
                    self.context.clone(),
                    Box::new(search_ctx.clone()),
                    observer.clone(),
                ))
                .unwrap();
        }
    }

    pub fn wait(&self) {
        for (worker, _) in &self.workers {
            worker.send(Message::Wait).unwrap();
        }
    }

    pub fn stop(&self) {
        self.context.time_manager.abort_now();
        self.wait();
    }

    pub fn quit(&mut self) {
        self.context.time_manager.abort_now();
        for (worker, handle) in self.workers.drain(..) {
            worker.send(Message::Quit).unwrap();
            handle.join().unwrap();
        }
    }

    pub fn clear(&self) {
        self.context.t_table.clear();
        for (worker, _) in &self.workers {
            worker.send(Message::Clear).unwrap();
        }
    }

    fn spawn_worker(thread_id: usize) -> (SyncSender<Message>, JoinHandle<()>) {
        let (tx, rx) = sync_channel(0);
        let mut thread_ctx = ThreadData::new_with_id(thread_id);
        let handle = thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                match msg {
                    Message::Search(shared_ctx, mut info, observer) => {
                        root(&shared_ctx, &mut thread_ctx, &mut info, &*observer);
                    }
                    Message::Wait => continue,
                    Message::Quit => break,
                    Message::Clear => {
                        thread_ctx.clear();
                    }
                }
            }
        });
        (tx, handle)
    }
}

pub trait SearchObserver: Send + Sync + 'static {
    fn node_counter(&self, _thread_id: usize) -> Arc<AtomicU64> {
        Arc::new(AtomicU64::new(0))
    }

    fn search_result(
        &self,
        _thread_id: usize,
        _root: &Board,
        _eval: Evaluation,
        _elapsed: Duration,
        _depth: u32,
        _sel_depth: u32,
        _nodes: u64,
        _pv: &[Option<Move>],
    ) {
    }

    fn result(&self, _board: &Board, _best_move: Move, _eval: Evaluation) {}
}

pub fn root(
    shared_ctx: &SharedCtx,
    thread_data: &mut ThreadData,
    info: &mut SearchCtx,
    observer: &dyn SearchObserver,
) -> Option<(Move, Evaluation)> {
    let mut thread_ctx = ThreadCtx {
        thread_id: thread_data.thread_id,
        ss: &mut thread_data.ss,
        history: &thread_data.history,
        killers: &mut thread_data.killer_moves,
        nodes: observer.node_counter(thread_data.thread_id),
        root_nodes: [[0; Square::NUM]; Square::NUM],
        window: Window::new(15, 45, 100, 9),
        eval: info.root.get_eval(),
        stm: info.root.board().side_to_move(),
        abort: false,
        tt_hits: 0,
        tt_misses: 0,
        sel_depth: 0,
        tm: &shared_ctx.time_manager,
        start: info.start,
    };

    let mut depth = 1;
    let mut abort = false;

    let mut result = None;
    'outer: loop {
        let mut fail_cnt = 0;
        thread_ctx.window.reset();
        loop {
            if abort {
                break 'outer;
            }
            let (alpha, beta) = if thread_ctx.eval.raw().abs() < 1000 && depth > 4 && fail_cnt < 10
            {
                thread_ctx.window.get()
            } else {
                (Evaluation::min(), Evaluation::max())
            };
            thread_ctx.sel_depth = 0;
            let score = search::search::<Pv>(
                &mut info.root,
                &mut thread_ctx,
                shared_ctx,
                0,
                depth,
                alpha,
                beta,
                false,
            );
            let nodes = thread_ctx.nodes();
            if depth > 1 && thread_ctx.abort {
                break 'outer;
            }
            thread_ctx.window.set(score);
            thread_ctx.eval = score;
            let root_move = thread_ctx.ss[0].pv[0].unwrap();
            let move_nodes = thread_ctx.root_nodes[root_move.from as usize][root_move.to as usize];
            shared_ctx.time_manager.deepen(
                thread_ctx.thread_id,
                depth,
                move_nodes,
                nodes,
                score,
                root_move,
            );
            if (score > alpha && score < beta) || score.is_mate() {
                abort = shared_ctx
                    .time_manager
                    .abort_deepening(info.start, depth, nodes);
                result = Some((root_move, score));
                break;
            } else {
                fail_cnt += 1;
                if score <= alpha {
                    thread_ctx.window.fail_low();
                } else {
                    thread_ctx.window.fail_high();
                }
            }
        }
        let root = &thread_ctx.ss[0];
        observer.search_result(
            thread_ctx.thread_id,
            info.root.board(),
            thread_ctx.eval,
            thread_ctx.start.elapsed(),
            depth,
            thread_ctx.sel_depth,
            thread_ctx.nodes(),
            &root.pv[..root.pv_len],
        );
        depth += 1;
        if depth > 1
            && shared_ctx
                .time_manager
                .abort_deepening(info.start, depth, thread_ctx.nodes())
        {
            break 'outer;
        }
    }
    if let Some((mv, eval)) = result
        && thread_data.thread_id == 0
    {
        observer.result(info.root.board(), mv, eval);
    }
    result
}
